// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// TODO move database_proxy out of the database module to better enforce
// separation of concerns via module-level field / method visibility

use mlua::prelude::*;
use uuid::Uuid;

use crate::database::{Database, Object, PropertyValue, Verb};
use crate::error::ErrorCode::*;
use crate::result::{err, ok, to_lua_result, Result};

use super::verb::{VerbArgs, VerbDesc, VerbInfo};

macro_rules! unwrap {
    ($lua:expr, $e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return err($lua, e),
        }
    };
}

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
    #[deprecated]
    pub fn parse_uuid_old(uuid: &str) -> LuaResult<Uuid> {
        Uuid::parse_str(&uuid).map_err(LuaError::external)
    }

    pub fn parse_uuid(uuid: &str) -> Result<Uuid> {
        Uuid::parse_str(&uuid)
            .map_err(|e| E_INVARG.make(format!("{:?} is not a valid UUID: {}", uuid, e)))
    }

    fn err_no_object(uuid: &str) -> LuaError {
        LuaError::external(format!("No object found with UUID {}", uuid))
    }

    #[deprecated]
    #[allow(deprecated)]
    fn get_object_old<'a>(&self, db: &'a Database, uuid: &str) -> LuaResult<&'a Object> {
        self.get_object_by_uuid_old(db, &Self::parse_uuid_old(uuid)?)
    }

    fn get_object<'a>(&self, db: &'a Database, uuid: &str) -> Result<&'a Object> {
        db.get(&Self::parse_uuid(uuid)?)
    }

    #[deprecated]
    #[allow(deprecated)]
    fn get_object_by_uuid_old<'a>(&self, db: &'a Database, uuid: &Uuid) -> LuaResult<&'a Object> {
        db.get_old(uuid).map_err(LuaError::external)
    }

    #[deprecated]
    #[allow(deprecated)]
    fn get_verb_old<'a>(
        &self,
        lock: &'a RwLockReadGuard<Database>,
        uuid: &str,
        desc: &VerbDesc,
    ) -> LuaResult<&'a Verb> {
        let object = self.get_object_old(&lock, &uuid)?;
        match desc {
            VerbDesc::Index(n) => {
                let verb = object
                    .verbs()
                    .get(n - 1)
                    .ok_or_else(|| LuaError::external("No such verby birby"))?;
                Ok(&verb)
            }
            _ => Err(LuaError::external(format!(
                "get_verb not implemented yet for {}",
                desc
            ))),
        }
    }

    #[deprecated]
    fn get_verb_mut_old<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
        desc: &VerbDesc,
    ) -> LuaResult<&'a mut Verb> {
        #[allow(deprecated)]
        let object = self.get_object_mut_old(lock, &uuid)?;
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

    #[deprecated]
    fn get_object_mut_old<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
    ) -> LuaResult<&'a mut Object> {
        #[allow(deprecated)]
        lock.get_mut_old(&Self::parse_uuid_old(&uuid)?)
            .map_err(LuaError::external)
    }

    fn get_object_mut<'a>(
        &self,
        lock: &'a mut RwLockWriteGuard<Database>,
        uuid: &str,
    ) -> Result<&'a mut Object> {
        #[allow(deprecated)]
        lock.get_mut(&Self::parse_uuid(&uuid)?)
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
    fn move_object<'lua>(&self, (what, to): (String, String)) -> Result<LuaValue<'lua>> {
        let mut lock = self.db.write().unwrap();
        lock.move_object(&Self::parse_uuid(&what)?, &Self::parse_uuid(&to)?)
            .map(|_| LuaValue::Nil)
    }

    fn chparent<'lua>(&self, (child, new_parent): (String, String)) -> Result<LuaValue<'lua>> {
        let mut lock = self.db.write().unwrap();
        let child_uuid = Self::parse_uuid(&child)?;
        let new_parent_uuid = Self::parse_uuid(&new_parent)?;
        lock.chparent(&child_uuid, &new_parent_uuid)
            .map(|_| LuaValue::Nil)
    }
}

impl LuaUserData for DatabaseProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("create", |lua, this, (parent, owner): (String, String)| {
            let (parent, owner) = {
                // Verify parent, owner exist
                let lock = this.db.read().unwrap();
                let parent = unwrap!(lua, this.get_object(&lock, &parent));
                let owner = unwrap!(lua, this.get_object(&lock, &owner));
                // TODO verify valid, fertile
                (*parent.uuid(), *owner.uuid())
            };
            let mut lock = this.db.write().unwrap();
            ok(lua, lock.create(&parent, &owner).to_string())
        });

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
                let object = unwrap!(lua, this.get_object_mut(&mut lock, &uuid));
                to_lua_result(
                    lua,
                    object
                        .set_property(&key, PropertyValue::from_lua(value, lua)?)
                        .map(|_| LuaValue::Nil),
                )
            },
        );

        methods.add_method(
            "get_property",
            |lua, this, (uuid, key): (String, String)| {
                let lock = this.db.read().unwrap();

                match lock.get_property(&unwrap!(lua, Self::parse_uuid(&uuid)), &key) {
                    Ok(None) => ok(lua, LuaValue::Nil),
                    Ok(Some(value)) => ok(lua, value),
                    Err(e) => err(lua, e),
                }
            },
        );

        methods.add_method("move", |lua, this, args: (String, String)| {
            to_lua_result(lua, this.move_object(args))
        });
        methods.add_method("chparent", |lua, this, args: (String, String)| {
            to_lua_result(lua, this.chparent(args))
        });

        methods.add_method(
            "add_verb",
            |lua, this, (uuid, info, args): (String, VerbInfo, VerbArgs)| {
                let mut lock = this.db.write().unwrap();
                let verb = Verb::new(info, args);
                unwrap!(lua, lock.get(verb.owner())); // To explode if the owner doesn't exist
                let object = unwrap!(lua, this.get_object_mut(&mut lock, &uuid));
                to_lua_result(lua, object.add_verb(verb).map(|_| LuaValue::Nil))
            },
        );

        methods.add_method(
            "has_verb_with_name",
            |_lua, this, (uuid, name): (String, String)| {
                let lock = this.db.read().unwrap();
                lock.has_verb_with_name(&Self::parse_uuid_old(&uuid)?, &name)
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
                this.get_verb_mut_old(&mut lock, &uuid, &desc)?.code = code;
                Ok(())
            },
        );

        methods.add_method(
            "resolve_verb",
            |lua, this, (uuid, name): (String, String)| {
                let lock = this.db.read().unwrap();
                let verb = lock
                    .resolve_verb(&Self::parse_uuid_old(&uuid)?, &name)
                    .map_err(LuaError::RuntimeError)?;
                verb.to_lua(lua)
            },
        );

        methods.add_method(
            "set_into_list",
            |_lua, this, (uuid, key, path, value): (String, String, Vec<usize>, PropertyValue)| {
                let mut lock = this.db.write().unwrap();
                let object = this.get_object_mut_old(&mut lock, &uuid)?;
                object
                    .set_into_list(&key, path, value)
                    .map_err(LuaError::RuntimeError)
            },
        );

        methods.add_method("valid", |_lua, this, (uuid,): (String,)| {
            let lock = this.db.read().unwrap();
            Ok(this.get_object_old(&lock, &uuid).is_ok())
        });

        methods.add_method("verbs", |_lua, this, (uuid,): (String,)| {
            let lock = this.db.read().unwrap();
            let object = this.get_object_old(&lock, &uuid)?;
            Ok(object.verb_names())
        });

        methods.add_method(
            "verb_info",
            |lua, this, (uuid, desc): (String, VerbDesc)| {
                let lock = this.db.read().unwrap();
                let verb = this.get_verb_old(&lock, &uuid, &desc)?;
                Ok(verb.info.to_lua(lua))
            },
        );

        methods.add_method(
            "verb_code",
            |_lua, this, (uuid, desc): (String, VerbDesc)| {
                let lock = this.db.read().unwrap();
                let verb = this.get_verb_old(&lock, &uuid, &desc)?;
                Ok(verb.code.clone())
            },
        );

        methods.add_method("checkpoint", |lua, _this, ()| {
            return err(lua, E_NACC.make("Not implemented yet"));
            #[allow(unreachable_code)]
            Ok(LuaValue::Nil)
            /*
            let lock = this.db.read().unwrap();
            saveload::checkpoint(&lock, &saveload::SaveloadConfig::default())
                .map_err(LuaError::external)
            */
        });

        methods.add_method("recycle", |lua, this, (uuid,): (String,)| {
            // TODO permission checks
            // Re-parent children to parent of self
            let db = this.db.read().unwrap();
            let uuid = Self::parse_uuid_old(&uuid)?;
            let obj = this.get_object_by_uuid_old(&db, &uuid)?;
            let parent_uuid_opt = *obj.parent();
            let children_uuids: Vec<_> = obj.children().iter().cloned().collect();
            let content_uuids: Vec<_> = obj.contents().iter().cloned().collect();
            let nothing_uuid = *db.nothing_uuid();
            drop(db);

            if let Some(parent_uuid) = parent_uuid_opt {
                for child_uuid in children_uuids {
                    // TODO drop .map_err when this moves into run_to_lua_result
                    this.chparent((child_uuid.to_string(), parent_uuid.to_string()))
                        .map_err(|e| LuaError::external(e.to_string()))?;
                }
            } else {
                // TODO do something here :)
            }

            // Move contents to S.nothing
            // TODO drop .map_err when this moves into run_to_lua_result
            for content_uuid in content_uuids {
                Self::move_object(this, (content_uuid.to_string(), nothing_uuid.to_string()))
                    .map_err(|e| LuaError::external(e.to_string()))?;
            }

            // TODO ownership quota

            // And actually delete the object
            let mut db = this.db.write().unwrap();
            db.delete(&uuid).map_err(LuaError::external)
        });

        methods.add_method("is_player", |_lua, this, (uuid,): (String,)| {
            let db = this.db.read().unwrap();
            db.is_player(&Self::parse_uuid_old(&uuid)?)
                .map_err(LuaError::external)
        });

        methods.add_method(
            "set_player_flag",
            |_lua, this, (uuid, val): (String, bool)| {
                // TODO check permissions
                let mut db = this.db.write().unwrap();
                db.set_player_flag(&Self::parse_uuid_old(&uuid)?, val)
                    .map_err(LuaError::external)
            },
        );

        methods.add_method("players", |_lua, this, ()| {
            let db = this.db.read().unwrap();
            Ok(db
                .players()
                .iter()
                .cloned()
                .map(|u| u.to_string())
                .collect::<Vec<_>>())
        });

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, (uuid,): (String,)| {
            this.make_object_proxy(lua, &Self::parse_uuid_old(&uuid)?)
        });
    }
}
