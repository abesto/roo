use rhai::plugin::*;
use rhai::{Dynamic, Engine, EvalAltResult};

use crate::{database::{SharedDatabase, ID}, error::Error};

macro_rules! api_functions {
    ($db_in:ident -> $db_out:ident, $engine:ident, $($name:ident($($args:tt)*) $b:block),*) => {
        $(
            let $db_out = $db_in.clone();
            $engine.register_fn(stringify!($name), move |$($args)*| $b);
        )*
    };
}

#[allow(unused_variables)]
pub fn register_api(engine: &mut Engine, database: SharedDatabase) {
    api_functions!(
        database -> db,
        engine,

        // Non-MOO / testing functions
        echo(s: &str) { s.to_string()},
        get_highest_object_number() { db.read().get_highest_object_number() },

        // Fundamental Operations on Objects
        // https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects
        create() { ObjectProxy::new(db.clone(), db.write().create()) },
        valid(obj: ObjectProxy) { db.read().valid(obj.id) },

        // Our version of #42 is O(42)
        O(id: ID) { ObjectProxy::new(db.clone(), id) }
    );

    // Errors
    engine
        .register_type::<Error>()
        .register_global_module(exported_module!(global_errors).into())
        .register_fn("to_string", |e: &mut Error| e.to_string())
        .register_fn("to_debug", |e: &mut Error| format!("{:?}", e));

    // ObjectProxy
    engine.register_type::<ObjectProxy>()
    .register_indexer_get_result(ObjectProxy::get_property)
    .register_indexer_set_result(ObjectProxy::set_property);
}

#[export_module]
pub mod global_errors {
    use crate::error::Error;

    pub const E_INVIND: &Error = &Error::E_INVIND;
    pub const E_PROPNF: &Error = &Error::E_PROPNF;
}


#[derive(Debug, Clone)]
pub struct ObjectProxy {
    db: SharedDatabase,
    id: ID,
}

impl ObjectProxy {
    fn new(db: SharedDatabase, id: ID) -> Self {
        Self { db, id }
    }

    fn get_property(&mut self, key: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        self.db.read().get_property_dynamic(self.id, key)
    }

    fn set_property(&mut self, key: &str, value: Dynamic) -> Result<(), Box<EvalAltResult>> {
        self.db.write().set_property_dynamic(self.id, key, value)
    }
}

impl std::fmt::Display for ObjectProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("#{}", self.id))
    }
}
