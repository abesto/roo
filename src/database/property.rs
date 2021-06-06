use mlua::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Property {
    String(String),
    Integer(LuaInteger),
    // TODO add Uuid, Vec<Uuid>; collapse all built-in properties into here
    // TODO maybe collapse verb storage into here
}

impl<'lua> FromLua<'lua> for Property {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(s) => s.to_str().map(|s| Property::String(s.to_string())),
            LuaValue::Integer(n) => Ok(Property::Integer(n)),
            _ => Err(LuaError::RuntimeError(format!(
                "Unsupported type for value {:?}",
                lua_value
            ))),
        }
    }
}

impl<'lua> ToLua<'lua> for Property {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            Property::String(s) => s.to_lua(lua),
            Property::Integer(n) => n.to_lua(lua),
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
        let p0 = Property::String(s.clone());

        let l = p0.clone().to_lua(&lua).unwrap();
        assert_eq!(l, s.to_lua(&lua).unwrap());

        let p1 = Property::from_lua(l, &lua).unwrap();
        assert_eq!(p0, p1);
    }

    #[test]
    fn test_integer() {
        let n = 4242;
        let lua = Lua::new();
        let p0 = Property::Integer(n);

        let l = p0.clone().to_lua(&lua).unwrap();
        assert_eq!(l, n.to_lua(&lua).unwrap());

        let p1 = Property::from_lua(l, &lua).unwrap();
        assert_eq!(p0, p1);
    }
}
