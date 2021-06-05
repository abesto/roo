// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;
use uuid::Uuid;

use crate::database::Database;

#[derive(Clone)]
pub struct DatabaseProxy {
    db: Arc<RwLock<Database>>,
}

impl DatabaseProxy {
    #[must_use]
    pub(crate) fn new(db: Arc<RwLock<Database>>) -> Self {
        Self { db }
    }

    pub fn parse_uuid(uuid: &str) -> LuaResult<Uuid> {
        Uuid::parse_str(&uuid).map_err(|e| LuaError::RuntimeError(e.to_string()))
    }

    fn err_no_object(uuid: &str) -> LuaError {
        LuaError::RuntimeError(format!("No object found with UUID {}", uuid))
    }

    fn make_object_proxy<'lua>(&self, lua: &'lua Lua, uuid: &Uuid) -> LuaResult<LuaTable<'lua>> {
        if !self.db.read().unwrap().contains_object(uuid) {
            return Err(Self::err_no_object(&uuid.to_string()));
        }
        let object_proxy: LuaTable = lua.globals().get("ObjectProxy")?;
        let o: LuaTable = object_proxy.call_method("new", (uuid.to_string(),))?;
        Ok(o)
    }
}

impl LuaUserData for DatabaseProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("create", |lua, this, ()| {
            let uuid = {
                let mut lock = this.db.write().unwrap();
                lock.create()
            };
            this.make_object_proxy(lua, &uuid)
        });

        methods.add_method_mut(
            "set_property",
            |_lua, this, (uuid, key, value): (String, String, String)| {
                {
                    let mut lock = this.db.write().unwrap();
                    lock.get_mut(&Self::parse_uuid(&uuid)?)
                        .ok_or_else(|| Self::err_no_object(&uuid))?
                        .properties
                        .insert(key, value);
                }
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "get_property",
            |lua, this, (uuid, key): (String, String)| {
                let lock = this.db.read().unwrap();
                let object = lock
                    .get(&Self::parse_uuid(&uuid)?)
                    .ok_or_else(|| Self::err_no_object(&uuid))?;

                if key == "location" {
                    return Ok(if let Some(location) = object.location {
                        location.to_string().to_lua(lua)?
                    } else {
                        LuaValue::Nil
                    });
                }

                if let Some(value) = object.properties.get(&key) {
                    value.clone().to_lua(lua)
                } else {
                    Ok(LuaValue::Nil)
                }
            },
        );

        methods.add_method("move", |_lua, this, (what, to): (String, String)| {
            let mut lock = this.db.write().unwrap();
            lock.move_object(&Self::parse_uuid(&what)?, &Self::parse_uuid(&to)?);
            Ok(LuaValue::Nil)
        });

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, (uuid,): (String,)| {
            this.make_object_proxy(lua, &Self::parse_uuid(&uuid)?)
        });
    }
}
