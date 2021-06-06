// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;

use crate::database::{Database, DatabaseProxy};
use uuid::Uuid;

pub struct World {
    db: Arc<RwLock<Database>>,
    system: Uuid,
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        let db = Database::new();
        let db_lock = Arc::new(RwLock::new(db));
        let system = {
            let mut lock = db_lock.write().unwrap();
            lock.create()
        };

        let o = Self {
            db: db_lock,
            system,
        };

        // RooCore
        o.lua()
            .load(include_str!("../lua/core.lua"))
            .set_name("core")
            .unwrap()
            .exec()
            .unwrap();

        o
    }

    pub fn lua(&self) -> Lua {
        let lua = Lua::new();

        let dbproxy = DatabaseProxy::new(Arc::clone(&self.db));
        {
            let globals = lua.globals();
            globals.set("db", dbproxy).unwrap();
            globals.set("system_uuid", self.system.to_string()).unwrap();
        }

        // API
        lua.load(include_str!("../lua/api.lua"))
            .set_name("api")
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
    use crate::database::PropertyValue;

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
                &PropertyValue::String("test-1".to_string()),
                o1.get_property("x").unwrap()
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

    #[test]
    fn starting_room() {
        let world = World::new();
        let lua1 = world.lua();
        let lua2 = world.lua();

        assert_eq!(
            lua1.load("system.uuid").eval::<String>().unwrap(),
            lua2.load("system.uuid").eval::<String>().unwrap(),
        );
    }

    #[test]
    fn do_login_command() {
        let world = World::new();
        let lua = world.lua();
        lua.load("system.do_login_command()").exec().unwrap();
    }

    #[test]
    fn call_parent() {
        let world = World::new();
        let lua = world.lua();
        assert_eq!(
            lua.load("system.starting_room.look")
                .eval::<LuaValue>()
                .unwrap()
                .type_name(),
            "function"
        );
    }
}
