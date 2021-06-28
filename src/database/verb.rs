use std::convert::TryFrom;
use std::fmt::Display;

use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::command::{Command, ParsedCommand};
use crate::database::Object;

use super::DatabaseProxy;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        Self::try_from(&s).map_err(LuaError::external)
    }
}

// TODO generalize various permission objects once more are added
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

impl ToString for VerbPermissions {
    fn to_string(&self) -> String {
        let mut s = String::new();
        if self.r {
            s += "r";
        }
        if self.w {
            s += "w";
        }
        if self.x {
            s += "x";
        }
        s
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerbDesc {
    Index(usize),
    Name(String),
}

impl<'lua> FromLua<'lua> for VerbDesc {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            LuaValue::Integer(n) => Ok(VerbDesc::Index(n as usize)),
            LuaValue::String(s) => Ok(VerbDesc::Name(s.to_str()?.to_string())),
            _ => Err(LuaError::external(format!(
                "Cannot build VerbDesc from {}",
                lua_value.type_name()
            ))),
        }
    }
}

impl Display for VerbDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbDesc::Index(n) => n.fmt(f),
            VerbDesc::Name(s) => s.fmt(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
                return Err(LuaError::external(
                    "verb-info table must have exactly three elements".to_string(),
                ));
            }
            Ok(Self {
                owner: DatabaseProxy::parse_uuid_old(&t.get::<LuaInteger, String>(1)?)?,
                perms: VerbPermissions::try_from(&t.get::<LuaInteger, String>(2)?)
                    .map_err(LuaError::external)?,
                names: t.get::<LuaInteger, Vec<String>>(3)?,
            })
        } else {
            Err(LuaError::external("verb-info must be a table".to_string()))
        }
    }
}

impl<'lua> ToLua<'lua> for &VerbInfo {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        if let LuaValue::Function(f) = lua
            .load("function (...) return {...} end")
            .eval::<LuaValue>()?
        {
            f.call((
                self.owner.to_string(),
                self.perms.to_string(),
                self.names.clone(),
            ))
        } else {
            unreachable!();
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        if value.is_empty() {
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
            } else {
                Ok(Self::Direct { dobj: t.get(1)? })
                // TODO impl: Self::Full
                /*
                Err(LuaError::external(
                    "only arg-less and dobj-only verbs are supported currently".to_string(),
                ))
                */
            }
        } else {
            Err(LuaError::external("verb-args must be a table".to_string()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verb {
    pub info: VerbInfo,
    pub args: VerbArgs,
    pub code: Vec<String>,
}

impl Verb {
    #[must_use]
    pub fn new(info: VerbInfo, args: VerbArgs) -> Self {
        Self {
            info,
            args,
            code: vec![],
        }
    }

    pub fn owner(&self) -> &Uuid {
        &self.info.owner
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
        let code = &format!(
            "function(this, ...)\nlocal args = {{...}}\n{}\nend",
            self.code.join("\n")
        );
        lua.load(code)
            .set_name(&self.names()[0])?
            .eval::<mlua::Value>()
    }
}
