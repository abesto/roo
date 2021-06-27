use mlua::ToLua;

use crate::error::Error;

pub fn ok<'lua, T>(lua: &'lua mlua::Lua, value: T) -> mlua::Result<mlua::Value<'lua>>
where
    T: ToLua<'lua>,
{
    let class: mlua::Table = lua.globals().get("Ok")?;
    let ctor: mlua::Function = class.get_metatable().unwrap().get("__call")?;
    ctor.call((class, value))
}

pub fn err(lua: &mlua::Lua, value: Error) -> mlua::Result<mlua::Value> {
    let class: mlua::Table = lua.globals().get("Err")?;
    let ctor: mlua::Function = class.get_metatable().unwrap().get("__call")?;
    ctor.call((class, value.to_lua(lua)))
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub fn result_to_lua<'lua, T>(
    lua: &'lua mlua::Lua,
    result: Result<T>,
) -> mlua::Result<mlua::Value<'lua>>
where
    T: mlua::ToLua<'lua> + std::fmt::Debug,
{
    match result {
        Ok(r) => r.to_lua(lua),
        Err(e) => e.to_lua(lua),
    }
}
