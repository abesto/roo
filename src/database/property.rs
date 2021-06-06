use std::collections::HashSet;

use mlua::prelude::*;
use uuid::Uuid;

use super::Verb;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PropertyValue {
    String(String),
    Integer(LuaInteger),
    Uuid(Uuid),
    UuidOpt(Option<Uuid>),
    Uuids(HashSet<Uuid>),
    Verb(Verb),
}

impl From<Uuid> for PropertyValue {
    fn from(uuid: Uuid) -> Self {
        Self::Uuid(uuid)
    }
}

impl From<Option<Uuid>> for PropertyValue {
    fn from(uuid: Option<Uuid>) -> Self {
        Self::UuidOpt(uuid)
    }
}

impl From<Verb> for PropertyValue {
    fn from(verb: Verb) -> Self {
        Self::Verb(verb)
    }
}

impl From<HashSet<Uuid>> for PropertyValue {
    fn from(value: HashSet<Uuid>) -> Self {
        Self::Uuids(value)
    }
}

impl<'lua> FromLua<'lua> for PropertyValue {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(s) => s.to_str().map(|s| PropertyValue::String(s.to_string())),
            LuaValue::Integer(n) => Ok(PropertyValue::Integer(n)),
            _ => Err(LuaError::RuntimeError(format!(
                "Unsupported type for value {:?}",
                lua_value
            ))),
        }
    }
}

impl<'lua> ToLua<'lua> for PropertyValue {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            PropertyValue::String(s) => s.to_lua(lua),
            PropertyValue::Integer(n) => n.to_lua(lua),
            PropertyValue::Uuid(id) => id.to_string().to_lua(lua),
            PropertyValue::Uuids(xs) => xs
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .to_lua(lua),
            PropertyValue::Verb(verb) => verb.to_lua(lua),
            PropertyValue::UuidOpt(o) => o
                .map(|uuid| uuid.to_string().to_lua(lua))
                .unwrap_or(Ok(LuaValue::Nil)),
        }
    }
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Property {
    pub value: PropertyValue,
}

impl<T> From<T> for Property
where
    T: Into<PropertyValue>,
{
    fn from(value: T) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let s = "test-str".to_string();
        let lua = Lua::new();
        let p0 = PropertyValue::String(s.clone());

        let l = p0.clone().to_lua(&lua).unwrap();
        assert_eq!(l, s.to_lua(&lua).unwrap());

        let p1 = PropertyValue::from_lua(l, &lua).unwrap();
        assert_eq!(p0, p1);
    }

    #[test]
    fn test_integer() {
        let n = 4242;
        let lua = Lua::new();
        let p0 = PropertyValue::Integer(n);

        let l = p0.clone().to_lua(&lua).unwrap();
        assert_eq!(l, n.to_lua(&lua).unwrap());

        let p1 = PropertyValue::from_lua(l, &lua).unwrap();
        assert_eq!(p0, p1);
    }
}
