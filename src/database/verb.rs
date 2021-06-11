use std::convert::TryFrom;

use mlua::prelude::*;
use uuid::Uuid;

use crate::command::{Command, ParsedCommand};
use crate::database::Object;

use super::DatabaseProxy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyOrThis {
    Any,
    This,
}

impl TryFrom<&String> for AnyOrThis {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "any" => Ok(AnyOrThis::Any),
            "this" => Ok(AnyOrThis::This),
            _ => Err(format!("invalid obj spec: {}", value)),
        }
    }
}

impl<'lua> FromLua<'lua> for AnyOrThis {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let s = String::from_lua(lua_value, lua)?;
        Self::try_from(&s).map_err(LuaError::RuntimeError)
    }
}

// TODO generalize various permission objects once more are added
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerbPermissions {
    pub r: bool,
    pub w: bool,
    pub x: bool,
}

impl VerbPermissions {
    fn new(r: bool, w: bool, x: bool) -> Self {
        Self { r, w, x }
    }
}

impl TryFrom<&String> for VerbPermissions {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let mut perms = VerbPermissions::new(false, false, false);
        for c in value.chars() {
            match c {
                'r' => perms.r = true,
                'w' => perms.w = true,
                'x' => perms.x = true,
                _ => {
                    return Err(format!(
                        "Verb permissions must only contain r, w, characters; found: {}",
                        c
                    ))
                }
            }
        }
        Ok(perms)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerbInfo {
    owner: Uuid,
    perms: VerbPermissions,
    names: Vec<String>,
}

impl VerbInfo {
    // TODO verify at creation time that names are all valid glob patterns
    pub fn new<S: ToString, P: Into<VerbPermissions>>(
        owner: Uuid,
        perms: P,
        names: Vec<S>,
    ) -> Self {
        Self {
            owner,
            perms: perms.into(),
            names: names.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<'lua> FromLua<'lua> for VerbInfo {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(t) = lua_value {
            if t.len()? != 3 {
                return Err(LuaError::RuntimeError(
                    "verb-info table must have exactly three elements".to_string(),
                ));
            }
            Ok(Self {
                owner: DatabaseProxy::parse_uuid(&t.get::<LuaInteger, String>(1)?)?,
                perms: VerbPermissions::try_from(&t.get::<LuaInteger, String>(2)?)
                    .map_err(LuaError::RuntimeError)?,
                names: t.get::<LuaInteger, Vec<String>>(3)?,
            })
        } else {
            Err(LuaError::RuntimeError(
                "verb-info must be a table".to_string(),
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerbArgs {
    NoArgs,
    Direct { dobj: AnyOrThis },
}

impl VerbArgs {
    pub fn no_args() -> Self {
        Self::NoArgs
    }
}

impl<S> TryFrom<&Vec<S>> for VerbArgs
where
    S: ToString,
{
    type Error = String;

    fn try_from(value: &Vec<S>) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            Ok(Self::NoArgs)
        } else if value.len() == 1 {
            let dobj = AnyOrThis::try_from(&value[0].to_string())?;
            Ok(Self::Direct { dobj })
        } else {
            Err("Can only handle arg-less and dobj-only verbs currently".to_string())
        }
    }
}

impl<'lua> FromLua<'lua> for VerbArgs {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Table(t) = lua_value {
            let len = t.len()?;
            if len == 0 {
                Ok(Self::NoArgs)
            } else if len == 1 {
                Ok(Self::Direct { dobj: t.get(1)? })
            } else {
                Err(LuaError::RuntimeError(
                    "only arg-less and dobj-only verbs are supported currently".to_string(),
                ))
            }
        } else {
            Err(LuaError::RuntimeError(
                "verb-args must be a table".to_string(),
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verb {
    pub info: VerbInfo,
    pub args: VerbArgs,
    pub code: String,
}

impl Verb {
    #[must_use]
    pub fn new(info: VerbInfo, args: VerbArgs) -> Self {
        Self {
            info,
            args,
            code: String::new(),
        }
    }

    pub fn names(&self) -> &Vec<String> {
        &self.info.names
    }

    pub fn name_matches(&self, needle: &str) -> bool {
        for name in self.names() {
            if let Ok(pattern) = glob::Pattern::new(name) {
                if pattern.matches(needle) {
                    return true;
                }
            }
        }
        false
    }

    pub fn matches(&self, _this: &Object, command: &Command) -> bool {
        match &self.args {
            VerbArgs::NoArgs => {
                matches!(command.parsed(), ParsedCommand::VerbNoArgs { verb } if self.name_matches(verb))
            }

            VerbArgs::Direct { dobj: _dobj } => {
                // TODO implement matching for dobj
                matches!(command.parsed(), ParsedCommand::VerbDirect { verb, direct: _direct } if self.name_matches(verb))
            }
        }
    }
}

impl<'lua> FromLuaMulti<'lua> for Verb {
    fn from_lua_multi(values: LuaMultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let (info, args) = <(VerbInfo, VerbArgs)>::from_lua_multi(values, lua)?;
        Ok(Self::new(info, args))
    }
}

impl<'lua> ToLua<'lua> for &Verb {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        // TODO memoize
        let code = &format!("function(this, args)\n{}\nend", self.code);
        lua.load(code)
            .set_name(&self.names()[0])?
            .eval::<mlua::Value>()
    }
}
