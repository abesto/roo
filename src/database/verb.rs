use std::convert::TryFrom;

use mlua::ToLua;

use crate::command::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyOrThis {
    Any,
    This,
}

impl TryFrom<&str> for AnyOrThis {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "any" => Ok(AnyOrThis::Any),
            "this" => Ok(AnyOrThis::This),
            _ => Err(format!("invalid obj spec: {}", value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerbSignature {
    NoArgs { name: String },
    Direct { name: String, dobj: AnyOrThis },
}

impl VerbSignature {
    pub fn name(&self) -> &str {
        match self {
            VerbSignature::NoArgs { name } => &name,
            VerbSignature::Direct { name, dobj: _dobj } => &name,
        }
    }

    pub fn matches(&self, command: &Command) -> bool {
        match self {
            VerbSignature::NoArgs { name } => {
                matches!(command, Command::VerbNoArgs { verb } if verb == name)
            }
            VerbSignature::Direct { name, dobj: _dobj } => {
                // TODO verify AnyOrThis::This match
                matches!(command, Command::VerbDirect { verb, direct: _direct } if verb == name)
            }
        }
    }
}

impl<S> TryFrom<&Vec<S>> for VerbSignature
where
    S: ToString,
{
    type Error = String;

    fn try_from(value: &Vec<S>) -> Result<Self, Self::Error> {
        if value.len() == 1 {
            Ok(Self::NoArgs {
                name: value[0].to_string(),
            })
        } else if value.len() == 2 {
            let dobj = AnyOrThis::try_from(value[1].to_string().as_str())?;
            Ok(Self::Direct {
                name: value[0].to_string(),
                dobj,
            })
        } else {
            Err("Can only handle arg-less and dobj-only verbs currently".to_string())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verb {
    pub(super) signature: VerbSignature,
    pub code: String,
}

impl Verb {
    #[must_use]
    fn new(signature: VerbSignature) -> Self {
        Self {
            signature,
            code: String::new(),
        }
    }

    pub fn name(&self) -> &str {
        self.signature.name()
    }
}

impl<S> TryFrom<&Vec<S>> for Verb
where
    S: ToString,
{
    type Error = String;

    fn try_from(value: &Vec<S>) -> Result<Self, Self::Error> {
        Ok(Self::new(VerbSignature::try_from(value)?))
    }
}

impl<'lua> ToLua<'lua> for &Verb {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        // TODO memoize
        let code = &format!("function(this, args)\n{}\nend", self.code);
        lua.load(code).eval::<mlua::Value>()
    }
}
