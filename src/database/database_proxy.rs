// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use mlua::prelude::*;
use uuid::Uuid;

use crate::database::{Database, Object, PropertyValue, Verb};
use std::convert::TryFrom;

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

    #[allow(dead_code)]
    fn get_object<'a>(
        &self,
        lock: &'a RwLockReadGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a Object> {
        lock.get(&Self::parse_uuid(&uuid)?)
            .map_err(LuaError::RuntimeError)
    }

    fn get_object_mut<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a mut Object> {
        lock.get_mut(&Self::parse_uuid(&uuid)?)
            .map_err(LuaError::RuntimeError)
    }

    fn make_object_proxy<'lua>(&self, lua: &'lua Lua, uuid: &Uuid) -> LuaResult<LuaTable<'lua>> {
        if !self.db.read().unwrap().contains_object(uuid) {
            return Err(Self::err_no_object(&uuid.to_string()));
        }
        let object_proxy: LuaTable = lua.globals().get("ObjectProxy")?;
        let o: LuaTable = object_proxy.call_method("new", (uuid.to_string(),))?;
        Ok(o)
    }

    fn validate_property(key: &str, value: &LuaValue) -> LuaResult<()> {
        let type_name = value.type_name();
        match key {
            "name" => {
                if type_name == "string" {
                    Ok(())
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "'name' property must be a string, found {}",
                        type_name
                    )))
                }
            }
            _ => Ok(()),
        }
    }
}

impl LuaUserData for DatabaseProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("create", |lua, this, ()| {
            let uuid = {
                let mut lock = this.db.write().unwrap();
                lock.create()
            };
            this.make_object_proxy(lua, &uuid)
        });

        methods.add_method(
            "set_property",
            |lua, this, (uuid, key, value): (String, String, LuaValue)| {
                Self::validate_property(&key, &value)?;
                {
                    let mut lock = this.db.write().unwrap();
                    let object = this.get_object_mut(&mut lock, &uuid)?;
                    object.set_property(&key, PropertyValue::from_lua(value, lua)?);
                }
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "get_property",
            |lua, this, (uuid, key): (String, String)| {
                let lock = this.db.read().unwrap();

                if let Some(value) = lock
                    .get_property(&Self::parse_uuid(&uuid)?, &key)
                    .map_err(LuaError::RuntimeError)?
                {
                    value.clone().to_lua(lua)
                } else {
                    Ok(LuaValue::Nil)
                }
            },
        );

        methods.add_method("move", |_lua, this, (what, to): (String, String)| {
            let mut lock = this.db.write().unwrap();
            lock.move_object(&Self::parse_uuid(&what)?, &Self::parse_uuid(&to)?)
                .map_err(LuaError::RuntimeError)?;
            Ok(LuaValue::Nil)
        });

        methods.add_method(
            "chparent",
            |_lua, this, (child, new_parent): (String, String)| {
                let mut lock = this.db.write().unwrap();
                lock.chparent(&Self::parse_uuid(&child)?, &Self::parse_uuid(&new_parent)?)
                    .map_err(LuaError::RuntimeError)
            },
        );

        methods.add_method(
            "add_verb",
            |_lua, this, (uuid, signature): (String, Vec<String>)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;

                let verb = Verb::try_from(&signature).map_err(LuaError::RuntimeError)?;

                if object.contains_verb(verb.name()) {
                    return Err(LuaError::RuntimeError(format!(
                        "Verb {} already exists on {}",
                        verb.name(),
                        uuid
                    )));
                }

                object.set_property(verb.name(), verb.clone());
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "set_verb_code",
            |_lua, this, (uuid, name, code): (String, String, String)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;

                if let Some(PropertyValue::Verb(verb)) = object.get_property_mut(&name) {
                    verb.code = code;

                    Ok(LuaValue::Nil)
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "No verb {}_on {}",
                        name, uuid
                    )))
                }
            },
        );

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, (uuid,): (String,)| {
            this.make_object_proxy(lua, &Self::parse_uuid(&uuid)?)
        });
    }
}
