use std::collections::HashSet;

use mlua::prelude::*;
use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PropertyValue {
    Boolean(bool),
    String(String),
    Integer(LuaInteger),
    Uuid(Uuid),
    UuidOpt(Option<Uuid>),
    Uuids(HashSet<Uuid>),
    List(Vec<PropertyValue>),
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        PropertyValue::Boolean(value)
    }
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

impl From<HashSet<Uuid>> for PropertyValue {
    fn from(value: HashSet<Uuid>) -> Self {
        Self::Uuids(value)
    }
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> Self {
        if let Some(uuid) = Uuid::parse_str(value).ok() {
            PropertyValue::Uuid(uuid)
        } else {
            PropertyValue::String(value.to_string())
        }
    }
}

impl<'lua> FromLua<'lua> for PropertyValue {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(s) => s.to_str().map(PropertyValue::from),
            LuaValue::Integer(n) => Ok(PropertyValue::Integer(n)),
            LuaValue::Table(t) => {
                let mut values: Vec<PropertyValue> = vec![];
                for i in 1..=t.len()? {
                    let v_lua = t.get(i)?;
                    let v = PropertyValue::from_lua(v_lua, &lua)?;
                    values.push(v);
                }
                Ok(PropertyValue::List(values))
            }
            LuaValue::Boolean(b) => Ok(PropertyValue::from(b)),
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
            PropertyValue::UuidOpt(o) => o
                .map(|uuid| uuid.to_string().to_lua(lua))
                .unwrap_or(Ok(LuaValue::Nil)),
            PropertyValue::List(ps) => ps
                .iter()
                .map(|p| p.clone().to_lua(lua))
                .collect::<LuaResult<Vec<LuaValue>>>()?
                .to_lua(lua),
            PropertyValue::Boolean(b) => b.to_lua(lua),
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
