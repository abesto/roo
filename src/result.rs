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

pub fn err_to_lua<'lua>(lua: &'lua mlua::Lua, value: Error) -> mlua::Result<mlua::Value<'lua>> {
    let class: mlua::Table = lua.globals().get("Err")?;
    let ctor: mlua::Function = class.get_metatable().unwrap().get("__call")?;
    ctor.call((class, value))
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn result_to_lua<'lua, T>(
    lua: &'lua mlua::Lua,
    result: Result<T>,
) -> mlua::Result<mlua::Value<'lua>>
where
    T: mlua::ToLua<'lua> + std::fmt::Debug,
{
    if result.is_ok() {
        result.unwrap().to_lua(lua)
    } else {
        result.unwrap_err().to_lua(lua)
    }
}
