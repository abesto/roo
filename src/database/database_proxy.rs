// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use mlua::prelude::*;
use uuid::Uuid;

use crate::database::{Database, Object, Property, Verb};
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

    fn get_object<'a>(
        &self,
        lock: &'a RwLockReadGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a Object> {
        lock.get(&Self::parse_uuid(&uuid)?)
            .ok_or_else(|| Self::err_no_object(&uuid))
    }

    fn get_object_mut<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a mut Object> {
        lock.get_mut(&Self::parse_uuid(&uuid)?)
            .ok_or_else(|| Self::err_no_object(&uuid))
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

                    if key == "location" {
                        match value {
                            LuaValue::String(s) => {
                                object.location = Some(Self::parse_uuid(s.to_str()?)?)
                            }
                            _ => {
                                return Err(LuaError::RuntimeError(
                                    "location property must be set to a string (containing a UUID)"
                                        .to_string(),
                                ))
                            }
                        }
                    } else {
                        object
                            .properties
                            .insert(key, Property::from_lua(value, lua)?);
                    }
                }
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "get_property",
            |lua, this, (uuid, key): (String, String)| {
                let lock = this.db.read().unwrap();
                let object = this.get_object(&lock, &uuid)?;

                if key == "location" {
                    return Ok(if let Some(location) = object.location {
                        location.to_string().to_lua(lua)?
                    } else {
                        LuaValue::Nil
                    });
                }

                if key == "contents" {
                    return object
                        .contents()
                        .iter()
                        .cloned()
                        .map(|uuid| uuid.to_string())
                        .collect::<Vec<_>>()
                        .to_lua(lua);
                }

                if let Some(value) = object.properties.get(&key) {
                    value.clone().to_lua(lua)
                } else if let Some(verb) = object.verbs.get(&key) {
                    verb.to_lua(lua)
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

        methods.add_method(
            "add_verb",
            |_lua, this, (uuid, signature): (String, Vec<String>)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;

                let verb = Verb::try_from(&signature).map_err(LuaError::RuntimeError)?;

                if object.verbs.contains_key(verb.name()) {
                    return Err(LuaError::RuntimeError(format!(
                        "Verb {} already exists on {}",
                        verb.name(),
                        uuid
                    )));
                }

                object.verbs.insert(verb.name().to_string(), verb);
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "set_verb_code",
            |_lua, this, (uuid, name, code): (String, String, String)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;

                if let Some(verb) = object.verbs.get_mut(&name) {
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
