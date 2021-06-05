// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;

use crate::database::{Database, DatabaseProxy};

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
        lua.load(include_str!("../lua/ObjectProxy.lua"))
            .set_name("ObjectProxy")
            .unwrap()
            .exec()
            .unwrap();

        // RooCore
        lua.load(include_str!("../lua/core.lua"))
            .set_name("core")
            .unwrap()
            .exec()
            .unwrap();

        lua
    }

    pub fn db(&self) -> Arc<RwLock<Database>> {
        self.db.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Property;

    #[test]
    fn set_properties() {
        let world = World::new();
        let lua = world.lua();
        let globals = lua.globals();
        let db = world.db();

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
            let o1 = db.get(&uuid).unwrap();
            assert_eq!(
                &Property::String("test-1".to_string()),
                o1.properties.get("x").unwrap()
            );
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
