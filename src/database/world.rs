// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;
use uuid::Uuid;

use crate::{
    database::{Database, DatabaseProxy},
    saveload::SaveloadConfig,
};

pub struct World {
    db: Arc<RwLock<Database>>,
    system_uuid: Uuid,
    needs_minimal_core: bool,
}

impl World {
    #[must_use]
    pub fn from_saveload_config(saveload_config: &SaveloadConfig) -> Self {
        let (db, needs_minimal_core) = match saveload_config.load() {
            Ok(db) => {
                println!("Database loading succeeded");
                (db, false)
            }
            Err(msg) => {
                println!("World failed to load DB: {}", msg);
                (Database::new(), true)
            }
        };

        Self::from_database(db, needs_minimal_core)
    }

    #[must_use]
    pub fn from_database(db: Database, needs_minimal_core: bool) -> Self {
        let system_uuid = *db.system_uuid();
        let db_lock = Arc::new(RwLock::new(db));

        Self {
            db: db_lock,
            system_uuid,
            needs_minimal_core,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn new() -> Self {
        Self::from_database(Database::new(), true)
    }

    pub fn lua(&mut self) -> Lua {
        let lua = unsafe { Lua::unsafe_new() };

        // globals
        let dbproxy = DatabaseProxy::new(Arc::clone(&self.db));
        {
            let globals = lua.globals();
            globals.set("db", dbproxy).unwrap();
            globals
                .set("system_uuid", self.system_uuid.to_string())
                .unwrap();
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

        for (module, module_code) in vec![
            ("init", include_str!("../lua/init.lua")),
            ("result", include_str!("../lua/result.lua")),
            ("moo", include_str!("../lua/moo.lua")),
            ("proxies", include_str!("../lua/proxies.lua")),
            ("core", include_str!("../lua/core.lua")),
            ("webclient", include_str!("../lua/webclient.lua")),
            ("final", include_str!("../lua/final.lua")),
        ] {
            if module == "core" {
                if !self.needs_minimal_core {
                    continue;
                } else {
                    self.needs_minimal_core = false;
                }
            }
            lua.load(module_code)
                .set_name(module)
                .unwrap()
                .exec()
                .unwrap();
        }

        lua
    }

    pub fn db(&self) -> Arc<RwLock<Database>> {
        self.db.clone()
    }
}
