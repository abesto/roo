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
        let lua = unsafe { Lua::unsafe_new() };

        // globals
        let dbproxy = DatabaseProxy::new(Arc::clone(&self.db));
        {
            let globals = lua.globals();
            globals.set("db", dbproxy).unwrap();
            globals.set("system_uuid", self.system.to_string()).unwrap();
        }

        // Penlight Lua library
        for (module, module_code) in vec![
            (
                "pl.compat",
                include_str!("../lua/Penlight-1.10.0/lua/pl/compat.lua"),
            ),
            (
                "pl.utils",
                include_str!("../lua/Penlight-1.10.0/lua/pl/utils.lua"),
            ),
            (
                "pl.app",
                include_str!("../lua/Penlight-1.10.0/lua/pl/app.lua"),
            ),
            (
                "pl.array2d",
                include_str!("../lua/Penlight-1.10.0/lua/pl/array2d.lua"),
            ),
            (
                "pl.class",
                include_str!("../lua/Penlight-1.10.0/lua/pl/class.lua"),
            ),
            (
                "pl.comprehension",
                include_str!("../lua/Penlight-1.10.0/lua/pl/comprehension.lua"),
            ),
            (
                "pl.config",
                include_str!("../lua/Penlight-1.10.0/lua/pl/config.lua"),
            ),
            (
                "pl.data",
                include_str!("../lua/Penlight-1.10.0/lua/pl/data.lua"),
            ),
            (
                "pl.Date",
                include_str!("../lua/Penlight-1.10.0/lua/pl/Date.lua"),
            ),
            (
                "pl.dir",
                include_str!("../lua/Penlight-1.10.0/lua/pl/dir.lua"),
            ),
            (
                "pl.file",
                include_str!("../lua/Penlight-1.10.0/lua/pl/file.lua"),
            ),
            (
                "pl.func",
                include_str!("../lua/Penlight-1.10.0/lua/pl/func.lua"),
            ),
            (
                "pl.import_into",
                include_str!("../lua/Penlight-1.10.0/lua/pl/import_into.lua"),
            ),
            (
                "pl.init",
                include_str!("../lua/Penlight-1.10.0/lua/pl/init.lua"),
            ),
            (
                "pl.input",
                include_str!("../lua/Penlight-1.10.0/lua/pl/input.lua"),
            ),
            (
                "pl.lapp",
                include_str!("../lua/Penlight-1.10.0/lua/pl/lapp.lua"),
            ),
            (
                "pl.lexer",
                include_str!("../lua/Penlight-1.10.0/lua/pl/lexer.lua"),
            ),
            (
                "pl.List",
                include_str!("../lua/Penlight-1.10.0/lua/pl/List.lua"),
            ),
            (
                "pl.luabalanced",
                include_str!("../lua/Penlight-1.10.0/lua/pl/luabalanced.lua"),
            ),
            (
                "pl.Map",
                include_str!("../lua/Penlight-1.10.0/lua/pl/Map.lua"),
            ),
            (
                "pl.MultiMap",
                include_str!("../lua/Penlight-1.10.0/lua/pl/MultiMap.lua"),
            ),
            (
                "pl.operator",
                include_str!("../lua/Penlight-1.10.0/lua/pl/operator.lua"),
            ),
            (
                "pl.OrderedMap",
                include_str!("../lua/Penlight-1.10.0/lua/pl/OrderedMap.lua"),
            ),
            (
                "pl.path",
                include_str!("../lua/Penlight-1.10.0/lua/pl/path.lua"),
            ),
            (
                "pl.permute",
                include_str!("../lua/Penlight-1.10.0/lua/pl/permute.lua"),
            ),
            (
                "pl.pretty",
                include_str!("../lua/Penlight-1.10.0/lua/pl/pretty.lua"),
            ),
            (
                "pl.seq",
                include_str!("../lua/Penlight-1.10.0/lua/pl/seq.lua"),
            ),
            (
                "pl.Set",
                include_str!("../lua/Penlight-1.10.0/lua/pl/Set.lua"),
            ),
            (
                "pl.sip",
                include_str!("../lua/Penlight-1.10.0/lua/pl/sip.lua"),
            ),
            (
                "pl.strict",
                include_str!("../lua/Penlight-1.10.0/lua/pl/strict.lua"),
            ),
            (
                "pl.stringio",
                include_str!("../lua/Penlight-1.10.0/lua/pl/stringio.lua"),
            ),
            (
                "pl.stringx",
                include_str!("../lua/Penlight-1.10.0/lua/pl/stringx.lua"),
            ),
            (
                "pl.tablex",
                include_str!("../lua/Penlight-1.10.0/lua/pl/tablex.lua"),
            ),
            (
                "pl.template",
                include_str!("../lua/Penlight-1.10.0/lua/pl/template.lua"),
            ),
            (
                "pl.test",
                include_str!("../lua/Penlight-1.10.0/lua/pl/test.lua"),
            ),
            (
                "pl.text",
                include_str!("../lua/Penlight-1.10.0/lua/pl/text.lua"),
            ),
            (
                "pl.types",
                include_str!("../lua/Penlight-1.10.0/lua/pl/types.lua"),
            ),
            (
                "pl.url",
                include_str!("../lua/Penlight-1.10.0/lua/pl/url.lua"),
            ),
            (
                "pl.xml",
                include_str!("../lua/Penlight-1.10.0/lua/pl/xml.lua"),
            ),
        ] {
            let code = format!(
                "package.preload[\"{}\"] = function () {} end",
                module, module_code
            );
            lua.load(&code).set_name(module).unwrap().exec().unwrap();
        }

        // API
        lua.load(include_str!("../lua/api.lua"))
            .set_name("api")
            .unwrap()
            .exec()
            .unwrap();

        // Webclient interface
        lua.load(include_str!("../lua/webclient.lua"))
            .set_name("webclient")
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
    fn get_parent_property() {
        let world = World::new();
        let lua = world.lua();

        let retval = lua
            .load(
                "
        root = db:create()
        root.x = 3

        sub = db:create()
        sub:chparent(root)

        return sub.x
        ",
            )
            .eval::<LuaInteger>()
            .unwrap();

        assert_eq!(3, retval);
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
        assert_eq!(
            lua1.load("system.starting_room.uuid")
                .eval::<String>()
                .unwrap(),
            lua2.load("system.starting_room.uuid")
                .eval::<String>()
                .unwrap(),
        );
    }

    #[test]
    fn do_login_command() {
        let world = World::new();
        let lua = world.lua();
        lua.load("system.do_login_command()").exec().unwrap();
    }

    #[test]
    fn look() {
        let world = World::new();
        let lua = world.lua();
        lua.load(
            "
        player = db:create()
        player:move(system.starting_room)

        o = db:create()
        o:move(player.location)

        system.starting_room:look()
        ",
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn call_parent() {
        let world = World::new();
        let lua = world.lua();
        assert_eq!(
            lua.load("system.starting_room:tell{\"whee\"}")
                .eval::<LuaValue>()
                .unwrap()
                .type_name(),
            "nil"
        );
    }

    #[test]
    fn set_into_list() {
        let world = World::new();
        let lua = world.lua();

        let uuid_str = lua
            .load("return db:create().uuid")
            .eval::<String>()
            .unwrap();
        let uuid = Uuid::parse_str(&uuid_str).unwrap();

        let retval = lua
            .load(
                &("local o = db[\"".to_string()
                    + &uuid_str
                    + "\"]
        o.l = {1, 2, {3, 4}}
        o.l[1] = 'foo'
        o.l[3][1] = 'bar'
        o.l[4] = 5
        table.insert(o.l, 6)
        table.insert(o.l[3], o)
        table.insert(o.l[3], o.uuid)
        return o.l._inner
        "),
            )
            .set_name("set_into_list-test")
            .unwrap()
            .eval::<PropertyValue>()
            .unwrap();

        assert_eq!(
            PropertyValue::List(vec![
                PropertyValue::String("foo".to_string()),
                PropertyValue::Integer(2),
                PropertyValue::List(vec![
                    PropertyValue::String("bar".to_string()),
                    PropertyValue::Integer(4),
                    PropertyValue::Uuid(uuid.clone()),
                    PropertyValue::Uuid(uuid.clone())
                ]),
                PropertyValue::Integer(5),
                PropertyValue::Integer(6),
            ]),
            retval
        );
    }
}
