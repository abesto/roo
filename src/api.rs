use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use rhai::{plugin::*, Array};
use rhai::{Dynamic, Engine};

use crate::{
    database::{PropertyInfo, PropertyPerms, SharedDatabase, ID},
    error::{Error, RhaiError, RhaiResult},
};

macro_rules! api_functions {
    ($db_in:ident, $db_out:ident, $engine:ident, { $(fn $name:ident($($args:tt)*) -> $r:ty $b:block)* }) => {
        $(
            let $db_out = $db_in.clone();
            $engine.register_result_fn(stringify!($name), move |$($args)*| -> RhaiResult<$r> { $b });
        )*
    };
}

#[allow(unused_variables)]
pub fn register_api(engine: &mut Engine, database: SharedDatabase) {
    api_functions!(database, db, engine, {
        // Non-MOO / testing functions
        fn echo(s: &str) -> String {
            Ok(s.to_string())
        }

        fn get_highest_object_number() -> ID {
            Ok(db.read().get_highest_object_number())
        }

        // Fundamental Operations on Objects
        // https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects

        fn create(parent: O, owner: O) -> O {
            Ok(O::new(db.clone(), db.write().create(parent.id, owner.id)))
        }

        fn create(parent: O) -> O {
            // TODO default owner to the programmer, once we have a concept of "logged in player"
            Err("Not implemented yet".into())
        }

        fn valid(obj: O) -> bool {
            Ok(db.read().valid(obj.id))
        }

        // Operations on Properties
        // https://www.sindome.org/moo-manual.html#operations-on-properties

        fn add_property(obj: O, name: &str, value: Dynamic, info: Array) -> () {
            db.write()
                .add_property(obj.id, name, value, info.try_into()?)
        }

        fn property_info(obj: O, name: &str) -> Array {
            let lock = db.read();
            let info = lock.property_info(obj.id, name)?;
            Ok(vec![
                Dynamic::from(O::new(db.clone(), info.owner)),
                Dynamic::from(info.perms.to_string()),
            ])
        }

        // Our version of #42 is O(42)
        fn O(id: ID) -> O {
            Ok(O::new(db.clone(), id))
        }
    });

    // Implement #0 object notation
    // can only use valid identifiers, so no #42. N42 is an OK shorthand for O(42)
    // (O42 looks dumb, and O0 is downright evil, so let's not use O as the prefix)
    let db = database.clone();
    engine.on_var(move |name, _, _| {
        if let Some(id_str) = name.strip_prefix("N") {
            if let Ok(id) = id_str.parse() {
                return Ok(Some(Dynamic::from(O::new(db.clone(), id))));
            }
        }
        Ok(None)
    });

    // Failed attempt: custom syntax
    /*
    let db = database.clone();
    engine.register_custom_syntax_raw(
        "#",
        |symbols, _| match symbols.len() {
            1 => Ok(Some("$int$".into())),
            2 => Ok(None),
            _ => unreachable!(),
        },
        false,
        move |_, inputs| {
            Ok(Dynamic::from(O::new(
                db.clone(),
                inputs[0].get_literal_value().unwrap(),
            )))
        },
    );
    */

    // Errors
    engine
        .register_type::<Error>()
        .register_global_module(exported_module!(global_errors).into())
        .register_fn("to_string", |e: &mut Error| e.to_string())
        .register_fn("to_debug", |e: &mut Error| format!("{:?}", e));

    // ObjectProxy
    engine
        .register_type::<O>()
        .register_indexer_get_result(O::get_property)
        .register_indexer_set_result(O::set_property)
        .register_fn("to_string", |o: &mut O| format!("{}", o))
        .register_fn("to_debug", |o: &mut O| format!("{}", o));
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

// We'll be using this a whole lot, so...
type O = ObjectProxy;

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
        f.write_fmt(format_args!("N{}", self.id))
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
        Self::from_str(&value.as_string().unwrap())
    }
}

impl FromStr for PropertyPerms {
    type Err = RhaiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

impl ToString for PropertyPerms {
    fn to_string(&self) -> String {
        let mut chars = vec![];
        if self.r {
            chars.push('r');
        }
        if self.w {
            chars.push('w');
        }
        if self.c {
            chars.push('c');
        }
        chars.into_iter().collect()
    }
}
