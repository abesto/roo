// TODO maybe switch to parking_lot::Mutex
use std::sync::{Arc, RwLock};

use mlua::prelude::*;
use uuid::Uuid;

use crate::{
    database::{Database, DatabaseProxy},
    saveload::SaveloadConfig,
};

macro_rules! load_pl_modules {
    ($lua:ident, $($m:literal),*) => {
        $(
            let module_code = include_str!(concat!("../lua/Penlight-1.10.0/lua/pl/", $m, ".lua"));
            let code = format!(
                "package.preload[\"pl.{}\"] = function () {} end",
                $m, module_code
            );
            $lua.load(&code).set_name($m).unwrap().exec().unwrap();
        )*
    };
}

macro_rules! load_roo_modules {
    ($lua:ident, $($m:literal),*) => {
        $(
            let module_code = include_str!(concat!("../lua/", $m, ".lua"));
            $lua.load(module_code).set_name($m).unwrap().exec().unwrap();
        )*
    }
}

macro_rules! load_verbs {
    ($lua:ident, $(($obj:literal, [$($verb:literal),*])),*) => {
        $($(

            let module_code = include_str!(concat!("../lua/core/", $obj, "/", $verb, ".lua"));
            let code = format!("{}:set_verb_code('{}', [[\n{}\n]]):unwrap()", $obj, $verb, module_code);
            let module = concat!("load_verbs:", $obj, ":", $verb);
            $lua.load(&code).set_name(module).unwrap().exec().unwrap();
        )*)*
    }
}

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
        // init?
        load_pl_modules!(
            lua,
            "compat",
            "utils",
            "class",
            "func",
            "import_into",
            "input",
            "lexer",
            "luabalanced",
            "List",
            "Map",
            "MultiMap",
            "OrderedMap",
            "Set",
            "operator",
            "permute",
            "pretty",
            "seq",
            "strict",
            "stringx",
            "tablex",
            "test",
            "text",
            "types",
            "url"
        );

        load_roo_modules!(lua, "init", "result", "moo", "proxies");
        if self.needs_minimal_core {
            load_roo_modules!(lua, "core");
            load_verbs!(
                lua,
                ("system", ["do_login_command"]),
                (
                    "S.code_utils",
                    ["short_prep", "full_prep", "toobj", "parse_verbref"]
                )
            );
            self.needs_minimal_core = false;
        }
        load_roo_modules!(lua, "webclient", "final");

        lua
    }

    pub fn db(&self) -> Arc<RwLock<Database>> {
        self.db.clone()
    }
}
