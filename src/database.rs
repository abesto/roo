use std::collections::HashMap;
// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
struct Object {
    uuid: Uuid,
    properties: HashMap<String, String>,
}

impl Object {
    #[must_use]
    fn new(uuid: Uuid) -> Self {
        Object {
            uuid,
            properties: HashMap::new(),
        }
    }
}

impl LuaUserData for Object {}

struct Database {
    objects: HashMap<Uuid, Object>,
}

impl Database {
    #[must_use]
    fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

#[derive(Clone)]
struct DatabaseProxy {
    db: Arc<RwLock<Database>>,
}

impl DatabaseProxy {
    #[must_use]
    fn new(db: Arc<RwLock<Database>>) -> Self {
        Self { db }
    }

    fn parse_uuid(uuid: &str) -> LuaResult<Uuid> {
        Uuid::parse_str(&uuid).map_err(|e| LuaError::RuntimeError(e.to_string()))
    }

    fn make_object_proxy<'lua>(&self, lua: &'lua Lua, uuid: &Uuid) -> LuaResult<LuaTable<'lua>> {
        if !self.db.read().unwrap().objects.contains_key(uuid) {
            return Err(LuaError::RuntimeError(format!(
                "No object found with UUID {}",
                uuid
            )));
        }
        let object_proxy: LuaTable = lua.globals().get("ObjectProxy")?;
        let o: LuaTable = object_proxy.call_method("new", (uuid.to_string(),))?;
        Ok(o)
    }
}

impl LuaUserData for DatabaseProxy {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("create", |lua, this, ()| {
            let uuid = Uuid::new_v4();

            {
                let mut lock = this.db.write().unwrap();
                lock.objects.insert(uuid, Object::new(uuid));
            }

            this.make_object_proxy(lua, &uuid)
        });

        methods.add_method_mut(
            "set_property",
            |_lua, this, (uuid, key, value): (String, String, String)| {
                {
                    let mut lock = this.db.write().unwrap();
                    lock.objects
                        .get_mut(&Self::parse_uuid(&uuid)?)
                        .ok_or(LuaError::RuntimeError(format!(
                            "No object found with UUID {}",
                            uuid
                        )))?
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
                let object =
                    lock.objects
                        .get(&Self::parse_uuid(&uuid)?)
                        .ok_or(LuaError::RuntimeError(format!(
                            "No object found with UUID {}",
                            uuid
                        )))?;
                if let Some(value) = object.properties.get(&key) {
                    value.clone().to_lua(lua)
                } else {
                    Ok(LuaValue::Nil)
                }
            },
        );

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, (uuid,): (String,)| {
            this.make_object_proxy(lua, &Self::parse_uuid(&uuid)?)
        });
    }
}

pub struct World {
    db: Arc<RwLock<Database>>,
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        let db = Database::new();
        let db_lock = Arc::new(RwLock::new(db));

        Self { db: db_lock }
    }

    pub fn lua(&self) -> Lua {
        let lua = Lua::new();

        let dbproxy = DatabaseProxy::new(Arc::clone(&self.db));
        {
            let globals = lua.globals();
            globals.set("db", dbproxy).unwrap();
        }

        // API
        lua.load(include_str!("lua/ObjectProxy.lua"))
            .set_name("ObjectProxy")
            .unwrap()
            .exec()
            .unwrap();

        lua
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_properties() {
        let world = World::new();
        let lua = world.lua();
        let globals = lua.globals();
        let db = world.db;

        // Step 1: create an object and set a property on it
        lua.load("o1 = db:create(); o1.x = \"test-1\"")
            .exec()
            .unwrap();

        {
            let db = db.read().unwrap();
            let o1_proxy: LuaTable = globals.get("o1").unwrap();
            assert_eq!("test-1", o1_proxy.get::<&str, String>("x").unwrap());

            let uuid =
                DatabaseProxy::parse_uuid(&o1_proxy.get::<&str, String>("uuid").unwrap()).unwrap();
            let o1 = db.objects.get(&uuid).unwrap();
            assert_eq!("test-1", o1.properties.get("x").unwrap());
        };

        // Step 2: get another reference to the same object, verify property
        lua.load("o2 = db[o1.uuid]").exec().unwrap();
        let o2_proxy: LuaTable = globals.get("o2").unwrap();
        assert_eq!("test-1", o2_proxy.get::<&str, String>("x").unwrap());

        // Step 3: set property on one reference, verify on the other
        lua.load("o1.x = \"test-2\"").exec().unwrap();
        let o2_proxy: LuaTable = globals.get("o2").unwrap();
        assert_eq!("test-2", o2_proxy.get::<&str, String>("x").unwrap());
    }
}
