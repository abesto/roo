use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use rhai::Array;
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

macro_rules! register_operators {
    ($engine:ident, $t:ty, $($op:tt),*) => {
        $(
            $engine.register_fn(
                stringify!($op),
                |a: $t, b: $t| a $op b
            );
        )*
    }
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
            Ok(O::new(db.write().create(parent.id, owner.id)))
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
                Dynamic::from(O::new(info.owner)),
                Dynamic::from(info.perms.to_string()),
            ])
        }

        // Our version of #42 is O(42)
        fn O(id: ID) -> O {
            Ok(O::new(id))
        }
    });

    // Failed attempt for #0 object notation: custom syntax
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
        .register_type_with_name::<Error>("Error")
        .register_fn("to_string", |e: &mut Error| e.to_string())
        .register_fn("to_debug", |e: &mut Error| format!("{:?}", e));
    register_operators!(engine, Error, ==, !=, <, >, <=, >=);

    // Custom variable resolvers
    let db = database.clone();
    engine.on_var(move |name, _, _| {
        // Error constants (like E_INVARG)
        if let Ok(e) = Error::from_str(name) {
            return Ok(Some(Dynamic::from(e)));
        }
        // N0 object notation (like #0 in Moo)
        if let Some(id_str) = name.strip_prefix("N") {
            if let Ok(id) = id_str.parse() {
                return Ok(Some(Dynamic::from(O::new(id))));
            }
        }
        // S
        Ok(None)
    });

    // ObjectProxy
    engine.register_type_with_name::<O>("Object");

    let db = database.clone();
    engine.register_get_result("name", move |o: &mut O| db.read().get_name(o.id));

    let db = database.clone();
    engine.register_set_result("name", move |o: &mut O, name: &str| {
        db.write().set_name(o.id, name)
    });

    let db = database.clone();
    engine.register_indexer_get_result(move |o: &mut O, prop: &str| {
        db.read().get_property_dynamic(o.id, prop)
    });

    let db = database.clone();
    engine.register_indexer_set_result(move |o: &mut O, prop: &str, val: Dynamic| {
        db.write().set_property_dynamic(o.id, prop, val)
    });

    let db = database.clone();
    engine.register_fn("to_string", |o: &mut O| format!("{}", o));

    let db = database.clone();
    engine.register_fn("to_debug", |o: &mut O| format!("{}", o));
    register_operators!(engine, O, ==, !=, <, >, <=, >=);

    // Spread assignment
    engine
        .register_custom_operator("lets", 20)
        .unwrap()
        .register_custom_syntax_raw(
            "lets",
            |symbols, look_ahead| {
                // lets ...
                if symbols.len() == 1 {
                    return Ok(Some("[".into()));
                }

                let last = symbols.last().unwrap().as_str();

                // lets []...
                if !symbols.contains(&"]".into()) {
                    if (last != "," && last != "=") && look_ahead == "]" {
                        return Ok(Some("]".into()));
                    }
                    let next = match last {
                        "," | "[" => "$ident$",
                        "=" => "$expr$",
                        "$expr$" => ",",
                        _ => match look_ahead {
                            "=" => "=",
                            _ => ",",
                        },
                    };
                    return Ok(Some(next.into()));
                }

                // lets [___] ...
                if last == "]" {
                    return Ok(Some("=".into()));
                }

                // lets [___] = ...
                if last == "=" {
                    return Ok(Some("$expr$".into()));
                }

                // All done
                Ok(None)
            },
            true,
            |context, inputs| -> RhaiResult<Dynamic> {
                #[derive(Debug)]
                struct Var {
                    name: String,
                    optional: bool,
                    rest: bool,
                    default: Option<Dynamic>,
                }
                let values: Array = context.eval_expression_tree(inputs.last().unwrap())?.cast();

                let mut vars: Vec<Var> = vec![];
                let mut rest: Option<String> = None;
                for input in &inputs[..inputs.len() - 1] {
                    if let Some(var) = input.get_variable_name() {
                        if let Some(stripped) = var.strip_prefix("OPT_") {
                            // OPT_variable, same as ?variable in Moo
                            vars.push(Var {
                                name: stripped.to_string(),
                                optional: true,
                                rest: false,
                                default: None,
                            });
                        } else if let Some(stripped) = var.strip_prefix("REST_") {
                            // REST_variable, same as @variable in Moo
                            if let Some(rest_name) = &rest {
                                return Err(format!(
                                    "Tried to make {} a REST_ variable, but {} is already that.",
                                    stripped, rest_name
                                )
                                .into());
                            }
                            vars.push(Var {
                                name: stripped.to_string(),
                                optional: false,
                                rest: true,
                                default: None,
                            });
                            rest = Some(stripped.to_string());
                        } else {
                            // If we don't have a REST_ or an OPT_ prefix, then we're
                            // a normal required variable
                            vars.push(Var {
                                name: var.to_string(),
                                optional: false,
                                rest: false,
                                default: None,
                            });
                        }
                    } else {
                        // If we get a thing that's not a variable, then (because of how)
                        // the grammar is defined) it MUST be the default value
                        // for the last variable we added.
                        let last = vars.last_mut().unwrap();
                        last.default = Some(context.eval_expression_tree(input)?);
                        last.optional = true;
                    }
                }

                let req_count = vars.iter().filter(|v| !v.optional && !v.rest).count();
                let opt_count = vars.iter().filter(|v| v.optional && !v.rest).count();

                println!("{:?}", vars);
                if values.len() < req_count {
                    bail!(Error::E_INVARG);
                }
                if values.len() > req_count + opt_count && rest.is_none() {
                    bail!(Error::E_INVARG);
                }

                let opt_matched = std::cmp::min(opt_count, values.len() - req_count);
                let mut opt_left = opt_matched;
                let mut val_i = 0;
                println!(
                    "values.len()={} req_count={} opt_count={} opt_matched={}",
                    values.len(),
                    req_count,
                    opt_count,
                    opt_matched
                );

                // Fill in all required vars and the optional ones
                // that got enough arguments / have defaults
                for var in &vars {
                    if !var.optional && !var.rest {
                        // Required vars are always filled from the input
                        let val = values[val_i].clone();
                        context.scope_mut().push(var.name.clone(), val);
                        val_i += 1;
                    } else if var.optional && !var.rest {
                        if opt_left > 0 {
                            // If there's enough items in the input, fill optional vars as well
                            let val = values[val_i].clone();
                            context.scope_mut().push(var.name.clone(), val);
                            opt_left -= 1;
                            val_i += 1;
                        } else if let Some(default) = &var.default {
                            // If we're out of input, but have a default for an optional var, then use the default
                            context.scope_mut().push(var.name.clone(), default.clone());
                        }
                    } else if var.rest {
                        // Capture the right number of values for the next variable
                        let rest_count = values.len() - req_count - opt_matched;
                        if rest_count > 0 {
                            let rest_values: Array = values[val_i..(val_i + rest_count)].into();
                            context.scope_mut().push(var.name.clone(), rest_values);
                            val_i += rest_count;
                        } else {
                            context.scope_mut().push(var.name.clone(), Array::new());
                        }
                    }
                }

                Ok(Dynamic::UNIT)
            },
        );
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct ObjectProxy {
    id: ID,
}

// We'll be using this a whole lot, so...
type O = ObjectProxy;

impl ObjectProxy {
    fn new(id: ID) -> Self {
        Self { id }
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
