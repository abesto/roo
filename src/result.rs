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

pub fn to_lua_result<'lua, T>(
    lua: &'lua mlua::Lua,
    result: Result<T>,
) -> mlua::Result<mlua::Value<'lua>>
where
    T: mlua::ToLua<'lua> + std::fmt::Debug,
{
    match result {
        Ok(r) => ok(lua, r),
        Err(e) => err(lua, e),
    }
}

pub fn run_to_lua_result<'lua, T, F>(lua: &'lua mlua::Lua, f: F) -> mlua::Result<mlua::Value<'lua>>
where
    T: mlua::ToLua<'lua> + std::fmt::Debug,
    F: FnOnce() -> Result<T>,
{
    to_lua_result(lua, f())
}
