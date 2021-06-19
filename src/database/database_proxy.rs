// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// TODO move database_proxy out of the database module to better enforce
// separation of concerns via module-level field / method visibility

use mlua::prelude::*;
use uuid::Uuid;

use crate::database::{Database, Object, PropertyValue, Verb};
use crate::saveload;
use crate::server::CONNDATA;

use super::verb::{VerbArgs, VerbDesc, VerbInfo};

#[derive(Clone)]
pub struct DatabaseProxy {
    db: Arc<RwLock<Database>>,
}

impl DatabaseProxy {
    #[must_use]
    pub(crate) fn new(db: Arc<RwLock<Database>>) -> Self {
        Self { db }
    }

    // TODO move this function out of DatabaseProxy, it is used widely
    pub fn parse_uuid(uuid: &str) -> LuaResult<Uuid> {
        Uuid::parse_str(&uuid).map_err(LuaError::external)
    }

    fn err_no_object(uuid: &str) -> LuaError {
        LuaError::external(format!("No object found with UUID {}", uuid))
    }

    fn get_object<'a>(&self, db: &'a Database, uuid: &str) -> LuaResult<&'a Object> {
        self.get_object_by_uuid(db, &Self::parse_uuid(uuid)?)
    }

    fn get_object_by_uuid<'a>(&self, db: &'a Database, uuid: &Uuid) -> LuaResult<&'a Object> {
        db.get(uuid).map_err(LuaError::external)
    }

    fn get_verb<'a>(
        &self,
        lock: &'a RwLockReadGuard<Database>,
        uuid: &str,
        desc: &VerbDesc,
    ) -> LuaResult<&'a Verb> {
        let object = self.get_object(&lock, &uuid)?;
        match desc {
            VerbDesc::Index(n) => {
                let verb = object
                    .verbs()
                    .get(n - 1)
                    .ok_or_else(|| LuaError::external("No such verby birby"))?;
                Ok(&verb)
            }
            _ => unimplemented!(),
        }
    }

    fn get_verb_mut<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
        desc: &VerbDesc,
    ) -> LuaResult<&'a mut Verb> {
        let object = self.get_object_mut(lock, &uuid)?;
        match desc {
            VerbDesc::Index(n) => {
                let verb = object
                    .verbs_mut()
                    .get_mut(n - 1)
                    .ok_or_else(|| LuaError::external("No such verby birby"))?;
                Ok(verb)
            }
            VerbDesc::Name(name) => object
                .verbs_mut()
                .iter_mut()
                .find(|v| v.name_matches(name))
                .ok_or_else(|| LuaError::external("No such verb by name eh")),
        }
    }

    fn get_object_mut<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a mut Object> {
        lock.get_mut(&Self::parse_uuid(&uuid)?)
            .map_err(LuaError::external)
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

impl DatabaseProxy {
    fn lmove(_lua: &Lua, this: &DatabaseProxy, (what, to): (String, String)) -> LuaResult<()> {
        let mut lock = this.db.write().unwrap();
        lock.move_object(&Self::parse_uuid(&what)?, &Self::parse_uuid(&to)?)
            .map_err(LuaError::external)
    }

    fn chparent(
        _lua: &Lua,
        this: &DatabaseProxy,
        (child, new_parent): (String, String),
    ) -> LuaResult<()> {
        let mut lock = this.db.write().unwrap();
        lock.chparent(&Self::parse_uuid(&child)?, &Self::parse_uuid(&new_parent)?)
            .map_err(LuaError::external)
    }
}

impl LuaUserData for DatabaseProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "create",
            |_lua, this, (parent, owner): (String, Option<String>)| {
                let (parent, owner) = {
                    // Verify parent, owner exist
                    let lock = this.db.read().unwrap();
                    let parent = this.get_object(&lock, &parent)?;
                    let owner = this.get_object(
                        &lock,
                        &owner.unwrap_or_else(|| CONNDATA.get().player_object.to_string()),
                    )?;
                    // TODO verify valid, fertile
                    (parent.uuid().clone(), owner.uuid().clone())
                };
                let mut lock = this.db.write().unwrap();
                Ok(lock.create(&parent, &owner).to_string())
            },
        );

        methods.add_method(
            "set_property",
            |lua, this, (uuid, key, value): (String, String, LuaValue)| {
                let errmsg_opt = match key.as_str() {
                    "location" => Some(".location cannot be set directly. Use what:move(where)"),
                    "contents" => Some(".contents cannot be set directly. Use what:move(where)"),
                    "children" => {
                        Some(".children cannot be set directly. Use child:chparent(new_parent)")
                    }
                    "parent" => {
                        Some(".parent cannot be set directly. Use child:chparent(new_parent)")
                    }
                    _ => None,
                };

                if let Some(errmsg) = errmsg_opt {
                    return Err(LuaError::external(errmsg));
                }

                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;
                object
                    .set_property(&key, PropertyValue::from_lua(value, lua)?)
                    .map_err(LuaError::external)?;
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "get_property",
            |lua, this, (uuid, key): (String, String)| {
                let lock = this.db.read().unwrap();

                if let Some(value) = lock
                    .get_property(&Self::parse_uuid(&uuid)?, &key)
                    .map_err(LuaError::external)?
                {
                    value.clone().to_lua(lua)
                } else {
                    Ok(LuaValue::Nil)
                }
            },
        );

        methods.add_method("move", DatabaseProxy::lmove);
        methods.add_method("chparent", DatabaseProxy::chparent);

        methods.add_method(
            "add_verb",
            |_lua, this, (uuid, info, args): (String, VerbInfo, VerbArgs)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;
                let verb = Verb::new(info, args);
                object.add_verb(verb).map_err(LuaError::RuntimeError)?;
                Ok(LuaValue::Nil)
            },
        );

        methods.add_method(
            "has_verb_with_name",
            |_lua, this, (uuid, name): (String, String)| {
                let lock = this.db.read().unwrap();
                lock.has_verb_with_name(&Self::parse_uuid(&uuid)?, &name)
                    .map_err(LuaError::RuntimeError)
            },
        );

        methods.add_method(
            "set_verb_code",
            |lua, this, (uuid, desc, code): (String, VerbDesc, Vec<String>)| {
                // Verify the code is at least mostly sane
                lua.load(&code.join("\n"))
                    .set_name(&format!("validate_verb_code {}:{}", uuid, desc))?
                    .into_function()?;

                // And write it
                let mut lock = this.db.write().unwrap();
                this.get_verb_mut(&mut lock, &uuid, &desc)?.code = code;
                Ok(())
            },
        );

        methods.add_method(
            "resolve_verb",
            |lua, this, (uuid, name): (String, String)| {
                let lock = this.db.read().unwrap();
                let verb = lock
                    .resolve_verb(&Self::parse_uuid(&uuid)?, &name)
                    .map_err(LuaError::RuntimeError)?;
                verb.to_lua(lua)
            },
        );

        methods.add_method(
            "set_into_list",
            |_lua, this, (uuid, key, path, value): (String, String, Vec<usize>, PropertyValue)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut(&mut lock, &uuid)?;
                object
                    .set_into_list(&key, path, value)
                    .map_err(LuaError::RuntimeError)
            },
        );

        methods.add_method("valid", |_lua, this, (uuid,): (String,)| {
            let lock = this.db.read().unwrap();
            Ok(this.get_object(&lock, &uuid).is_ok())
        });

        methods.add_method("verbs", |_lua, this, (uuid,): (String,)| {
            let lock = this.db.read().unwrap();
            let object = this.get_object(&lock, &uuid)?;
            Ok(object.verb_names())
        });

        methods.add_method(
            "verb_info",
            |_lua, this, (uuid, desc): (String, VerbDesc)| {
                let lock = this.db.read().unwrap();
                let verb = this.get_verb(&lock, &uuid, &desc)?;
                Ok(verb.info.clone())
            },
        );

        methods.add_method(
            "verb_code",
            |_lua, this, (uuid, desc): (String, VerbDesc)| {
                let lock = this.db.read().unwrap();
                let verb = this.get_verb(&lock, &uuid, &desc)?;
                Ok(verb.code.clone())
            },
        );

        methods.add_method("checkpoint", |_lua, this, ()| {
            let lock = this.db.read().unwrap();
            saveload::checkpoint(&lock, &saveload::SaveloadConfig::default())
                .map_err(LuaError::external)
        });

        methods.add_method("recycle", |lua, this, (uuid,): (String,)| {
            // TODO permission checks
            // Re-parent children to parent of self
            let db = this.db.read().unwrap();
            let uuid = Self::parse_uuid(&uuid)?;
            let obj = this.get_object_by_uuid(&db, &uuid)?;
            let parent_uuid_opt = obj.parent().clone();
            let children_uuids: Vec<_> = obj.children().iter().cloned().collect();
            let content_uuids: Vec<_> = obj.contents().iter().cloned().collect();
            let nothing_uuid = db.nothing_uuid().clone();
            drop(db);

            if let Some(parent_uuid) = parent_uuid_opt {
                for child_uuid in children_uuids {
                    Self::chparent(lua, this, (child_uuid.to_string(), parent_uuid.to_string()))?;
                }
            } else {
                // TODO do something here :)
            }

            // Move contents to S.nothing
            for content_uuid in content_uuids {
                Self::lmove(
                    lua,
                    this,
                    (content_uuid.to_string(), nothing_uuid.to_string()),
                )?;
            }

            // TODO ownership quota

            // And actually delete the object
            let mut db = this.db.write().unwrap();
            db.delete(&uuid).map_err(LuaError::external)
        });

        methods.add_method("is_player", |_lua, this, (uuid,): (String,)| {
            let db = this.db.read().unwrap();
            db.is_player(&Self::parse_uuid(&uuid)?)
                .map_err(LuaError::external)
        });

        methods.add_method(
            "set_player_flag",
            |_lua, this, (uuid, val): (String, bool)| {
                // TODO check permissions
                let mut db = this.db.write().unwrap();
                db.set_player_flag(&Self::parse_uuid(&uuid)?, val)
                    .map_err(LuaError::external)
            },
        );

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, (uuid,): (String,)| {
            this.make_object_proxy(lua, &Self::parse_uuid(&uuid)?)
        });
    }
}
