use std::convert::{TryFrom, TryInto};

use rhai::{plugin::*, Array};
use rhai::{Dynamic, Engine};

use crate::{
    database::{PropertyInfo, PropertyPerms, SharedDatabase, ID},
    error::{Error, RhaiError, RhaiResult},
};

macro_rules! api_functions {
    ($db_in:ident -> $db_out:ident, $engine:ident, $($name:ident($($args:tt)*) -> $r:ty $b:block),*) => {
        $(
            let $db_out = $db_in.clone();
            $engine.register_result_fn(stringify!($name), move |$($args)*| -> RhaiResult<$r> { $b });
        )*
    };
}

#[allow(unused_variables)]
pub fn register_api(engine: &mut Engine, database: SharedDatabase) {
    api_functions!(
        database -> db,
        engine,

        // Non-MOO / testing functions
        echo(s: &str) -> String { Ok(s.to_string()) },
        get_highest_object_number() -> ID { Ok(db.read().get_highest_object_number()) },

        // Fundamental Operations on Objects
        // https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects
        create() -> ObjectProxy { Ok(ObjectProxy::new(db.clone(), db.write().create())) },
        valid(obj: ObjectProxy) -> bool { Ok(db.read().valid(obj.id)) },

        // Operations on Properties
        // https://www.sindome.org/moo-manual.html#operations-on-properties
        add_property(obj: ObjectProxy, name: &str, value: Dynamic, info: Array) -> () {
            db.write().add_property(obj.id, name, value, info.try_into()?)
        },

        // Our version of #42 is O(42)
        O(id: ID) -> ObjectProxy { Ok(ObjectProxy::new(db.clone(), id)) }
    );

    // Errors
    engine
        .register_type::<Error>()
        .register_global_module(exported_module!(global_errors).into())
        .register_fn("to_string", |e: &mut Error| e.to_string())
        .register_fn("to_debug", |e: &mut Error| format!("{:?}", e));

    // ObjectProxy
    engine
        .register_type::<ObjectProxy>()
        .register_indexer_get_result(ObjectProxy::get_property)
        .register_indexer_set_result(ObjectProxy::set_property);
}

#[export_module]
pub mod global_errors {
    use crate::error::Error;

    pub const E_INVIND: &Error = &Error::E_INVIND;
    pub const E_PROPNF: &Error = &Error::E_PROPNF;
    pub const E_INVARG: &Error = &Error::E_INVARG;
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

    fn get_property(&mut self, key: &str) -> RhaiResult<Dynamic> {
        self.db.read().get_property_dynamic(self.id, key)
    }

    fn set_property(&mut self, key: &str, value: Dynamic) -> RhaiResult<()> {
        self.db.write().set_property_dynamic(self.id, key, value)
    }
}

impl std::fmt::Display for ObjectProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("#{}", self.id))
    }
}

impl TryFrom<Array> for PropertyInfo {
    type Error = RhaiError;

    fn try_from(value: Array) -> Result<Self, Self::Error> {
        if value.len() < 2 || value.len() > 3 {
            bail!(Error::E_INVARG);
        }

        let owner = match value[0].clone().try_cast::<ObjectProxy>() {
            None => bail!(Error::E_INVARG),
            Some(obj) => obj.id,
        };

        let perms: PropertyPerms = value[1].clone().try_into()?;

        let new_name = match value.get(2) {
            None => None,
            Some(d) => {
                if d.is::<String>() {
                    Some(d.clone().as_string()?)
                } else {
                    bail!(Error::E_INVARG)
                }
            }
        };

        Ok(Self::new(owner, perms, new_name))
    }
}

impl TryFrom<Dynamic> for PropertyPerms {
    type Error = RhaiError;

    fn try_from(value: Dynamic) -> Result<Self, Self::Error> {
        if !value.is::<String>() {
            bail!(Error::E_INVARG);
        }
        let s = value.as_string()?;
        let mut r = false;
        let mut w = false;
        let mut c = false;
        for char in s.chars() {
            match char {
                'r' => r = true,
                'w' => w = true,
                'c' => c = true,
                _ => bail!(Error::E_INVARG),
            }
        }
        Ok(PropertyPerms::new(r, w, c))
    }
}
