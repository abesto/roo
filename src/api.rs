use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use rand::Rng;
use rhai::Array;
use rhai::{Dynamic, Engine};
use sha2::{Digest, Sha512};
use strum::EnumMessage;

use crate::{
    database::{PropertyInfo, PropertyPerms, SharedDatabase, ID},
    error::{
        Error::{self, *},
        RhaiError, RhaiResult,
    },
    task_context::TASK_CONTEXT,
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
        fn get_highest_object_number() -> ID {
            Ok(db.read().get_highest_object_number())
        }

        // Manipulating MOO Values / General Operations Applicable to all Values
        // https://www.sindome.org/moo-manual.html#general-operations-applicable-to-all-values

        fn tostr(x: Array) -> String {
            Ok(x.iter()
                .map(|d| {
                    if d.is::<Array>() {
                        return Ok("[list]".to_string());
                    } else if d.is::<String>() {
                        return Ok(d.clone_cast::<String>());
                    } else if d.is::<Error>() {
                        return Ok(d.clone_cast::<Error>().get_message().unwrap().to_string());
                    } else if d.is::<rhai::Map>() {
                        let map = d.clone_cast::<rhai::Map>();
                        if map
                            .get("error_marker")
                            .map(|m| m.clone().try_cast::<bool>())
                            == Some(Some(true))
                        {
                            return Ok(map["message"].clone_cast::<String>());
                        }
                    }
                    toliteral(d.clone())
                })
                .collect::<RhaiResult<Vec<_>>>()?
                .join(""))
        }

        // toliteral is recursive, so implemented in a standalone function
        // toint is used in toobj, so implemented in standalone functions

        fn toobj(o: O) -> O {
            Ok(o)
        }
        fn toobj(s: &str) -> O {
            let trimmed_0 = s.trim_start();
            let trimmed_1 = trimmed_0.strip_prefix('#').unwrap_or(trimmed_0);
            let trimmed_2 = trimmed_1.trim_start();
            str_toint(trimmed_2).map(O::new)
        }
        fn toobj(id: rhai::INT) -> O {
            Ok(O::new(id))
        }
        fn toobj(f: rhai::FLOAT) -> O {
            float_toint(f).map(O::new)
        }
        fn toobj(d: Dynamic) -> O {
            bail!(E_TYPE)
        }

        fn tofloat(f: rhai::FLOAT) -> rhai::FLOAT {
            Ok(f)
        }
        fn tofloat(i: rhai::INT) -> rhai::FLOAT {
            Ok(i as rhai::FLOAT)
        }
        fn tofloat(o: O) -> rhai::FLOAT {
            Ok(o.id as rhai::FLOAT)
        }
        // tofloat(s: &str) implemented in a standalone function because
        // it's used in toint
        fn tofloat(d: Dynamic) -> rhai::FLOAT {
            bail!(E_TYPE)
        }

        fn value_bytes(d: Dynamic) -> rhai::INT {
            Ok(std::mem::size_of_val(&d) as rhai::INT)
        }

        fn value_hash(d: Dynamic) -> String {
            let literal = toliteral(d)?;
            string_hash(&literal)
        }

        // Manipulating MOO Values / Operations on Strings
        // https://www.sindome.org/moo-manual.html#operations-on-

        // string_hash broken out to be used in value_hash

        // Fundamental Operations on Objects
        // https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects

        fn create(parent: O, owner: O) -> O {
            let id = TASK_CONTEXT.with(|context| {
                db.write()
                    .create(parent.id, Some(owner.id), context.read().task_perms)
            })?;
            Ok(O::new(id))
        }

        fn create(parent: O) -> O {
            let id = TASK_CONTEXT.with(|context| {
                db.write()
                    .create(parent.id, None, context.read().task_perms)
            })?;
            Ok(O::new(id))
        }

        fn parent(o: O) -> O {
            Ok(O::new(db.read().parent(o.id)))
        }

        fn chparent(o: O, parent: O) -> () {
            TASK_CONTEXT.with(|context| {
                db.write()
                    .chparent(o.id, parent.id, context.read().task_perms)
            })
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

        // Operations on Numbers
        // https://www.sindome.org/moo-manual.html#operations-on-numbers

        fn random(m: rhai::INT) -> rhai::INT {
            Ok(rand::thread_rng().gen_range(1..=m))
        }
        fn random() -> rhai::INT {
            Ok(rand::thread_rng().gen_range(1..=rhai::INT::MAX))
        }

        fn min(ns: Array) -> Dynamic {
            if ns.is_empty() {
                bail!(E_INVARG);
            }
            if ns[1..].iter().any(|n| n.type_name() != ns[0].type_name()) {
                bail!(E_TYPE);
            }
            if ns[0].is::<rhai::INT>() {
                Ok(ns
                    .iter()
                    .map(|n| n.clone_cast::<rhai::INT>())
                    .reduce(std::cmp::min)
                    .unwrap()
                    .into())
            } else if ns[0].is::<rhai::FLOAT>() {
                Ok(ns
                    .iter()
                    .map(|n| n.clone_cast::<rhai::FLOAT>())
                    .reduce(|a, b| a.min(b))
                    .unwrap()
                    .into())
            } else {
                bail!(E_TYPE)
            }
        }

        fn max(ns: Array) -> Dynamic {
            if ns.is_empty() {
                bail!(E_INVARG);
            }
            if ns[1..].iter().any(|n| n.type_name() != ns[0].type_name()) {
                bail!(E_TYPE);
            }
            if ns[0].is::<rhai::INT>() {
                Ok(ns
                    .iter()
                    .map(|n| n.clone_cast::<rhai::INT>())
                    .reduce(std::cmp::max)
                    .unwrap()
                    .into())
            } else if ns[0].is::<rhai::FLOAT>() {
                Ok(ns
                    .iter()
                    .map(|n| n.clone_cast::<rhai::FLOAT>())
                    .reduce(|a, b| a.max(b))
                    .unwrap()
                    .into())
            } else {
                bail!(E_TYPE)
            }
        }

        fn abs(n: rhai::INT) -> rhai::INT {
            Ok(n.abs())
        }
        fn abs(n: rhai::FLOAT) -> rhai::FLOAT {
            Ok(n.abs())
        }
        fn abs(d: Dynamic) -> Dynamic {
            bail!(E_TYPE)
        }

        // Built-in Functions / Manipulating Objects / MOO-Code Evaluation and Task Manipulation
        // https://www.sindome.org/moo-manual.html#moo-code-evaluation-and-task-management

        fn set_task_perms(who: O) -> () {
            TASK_CONTEXT.with(|context| {
                context.write().task_perms = who.id;
            });
            Ok(())
        }
    });

    // toliteral is recursive, so we need a standalone function definition first
    fn toliteral(x: Dynamic) -> RhaiResult<String> {
        Ok(if x.is::<O>() {
            format!("#{}", x.cast::<O>().id)
        } else if x.is::<String>() {
            format!("{:?}", x.cast::<String>())
        } else if x.is::<Error>() {
            format!("{}", x.cast::<Error>())
        } else if x.is::<Array>() {
            format!(
                "[{}]",
                x.cast::<Array>()
                    .iter()
                    .cloned()
                    .map(toliteral)
                    .collect::<RhaiResult<Vec<String>>>()?
                    .join(", "),
            )
        } else {
            x.to_string()
        })
    }
    engine.register_result_fn("toliteral", toliteral);

    fn str_tofloat(s: &str) -> RhaiResult<rhai::FLOAT> {
        Ok(s.split_whitespace()
            .collect::<String>()
            .parse::<rhai::FLOAT>()
            .unwrap_or(0.0))
    }
    engine.register_result_fn("tofloat", str_tofloat);

    // toint implementations, broken out into actual functions
    // so that they can be used in toobj
    fn int_toint(i: rhai::INT) -> RhaiResult<rhai::INT> {
        Ok(i)
    }
    engine.register_result_fn("toint", int_toint);

    fn float_toint(f: rhai::FLOAT) -> RhaiResult<rhai::INT> {
        Ok(f as rhai::INT)
    }
    engine.register_result_fn("toint", float_toint);

    fn object_toint(o: O) -> RhaiResult<rhai::INT> {
        Ok(o.id)
    }
    engine.register_result_fn("toint", object_toint);

    fn str_toint(s: &str) -> RhaiResult<rhai::INT> {
        str_tofloat(s).map(|f| f as rhai::INT)
    }
    engine.register_result_fn("toint", str_toint);

    fn dynamic_toint(d: Dynamic) -> RhaiResult<rhai::INT> {
        bail!(E_TYPE)
    }
    engine.register_result_fn("toint", dynamic_toint);

    // #0 object notation
    let db = database.clone();
    engine.register_custom_operator("#", 255).unwrap();
    engine.register_custom_syntax_raw(
        "#",
        |symbols, lookahead| match symbols.len() {
            1 if lookahead == "-" => Ok(Some("$symbol$".into())),
            1 => Ok(Some("$int$".into())),
            2 if symbols[1] == "-" => Ok(Some("$int$".into())),
            2 => Ok(None),
            3 => Ok(None),
            _ => unreachable!(),
        },
        false,
        move |_, inputs| {
            let id = if inputs.len() == 2 {
                assert_eq!(
                    inputs[0]
                        .get_literal_value::<rhai::ImmutableString>()
                        .unwrap(),
                    "-"
                );
                -inputs[1].get_literal_value::<ID>().unwrap()
            } else {
                inputs[0].get_literal_value::<ID>().unwrap()
            };
            Ok(Dynamic::from(O::new(id)))
        },
    );

    // $nothing corified references
    let db = database.clone();
    engine.register_custom_operator("$", 160).unwrap();
    engine.register_custom_syntax_raw(
        "$",
        |symbols, _| match symbols.len() {
            1 => Ok(Some("$ident$".into())),
            2 => Ok(None),
            _ => unreachable!(),
        },
        false,
        move |_, inputs| {
            let prop = inputs[0].get_variable_name().unwrap();
            db.read().get_property_dynamic(0, prop)
        },
    );

    fn string_hash(s: &str) -> RhaiResult<String> {
        let mut hasher = Sha512::new();
        hasher.update(s);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    engine.register_result_fn("string_hash", string_hash);

    // Errors
    engine
        .register_type_with_name::<Error>("Error")
        .register_fn("to_string", |e: &mut Error| e.to_string())
        .register_fn("to_debug", |e: &mut Error| format!("{:?}", e));
    register_operators!(engine, Error, ==, !=, <, >, <=, >=);

    // Custom variable resolvers
    let db = database.clone();
    engine.on_var(move |name, _, context| {
        // Error constants (like E_INVARG)
        if let Ok(e) = Error::from_str(name) {
            return Ok(Some(Dynamic::from(e)));
        }
        // Failed attempt at N0 object notation (like #0 in Moo)
        // Problem: returned value is constant, and even its fields are read-only, so you can't N2.f = true
        /*
        if let Some(id_str) = name.strip_prefix('N') {
            if let Ok(id) = id_str.parse() {
                return Ok(Some(Dynamic::from(O::new(id))));
            }
        }
        */
        // Cnothing corified notation (like $nothing in Moo)
        if let Some(prop) = name.strip_prefix('C') {
            return db.read().get_property_dynamic(0, prop).map(Some);
        }
        Ok(None)
    });

    // ObjectProxy
    engine.register_type_with_name::<O>("Object");

    // TODO: generate built-in property accessors with a macro

    // built-in property: name
    let db = database.clone();
    engine.register_get_result("name", move |o: &mut O| db.read().get_name(o.id));
    let db = database.clone();
    engine.register_set_result("name", move |o: &mut O, name: &str| {
        db.write().set_name(o.id, name)
    });

    // built-in property: f
    let db = database.clone();
    engine.register_get_result("f", move |o: &mut O| db.read().is_fertile(o.id));
    let db = database.clone();
    engine.register_set_result("f", move |o: &mut O, f: bool| {
        db.write().set_fertile(o.id, f)
    });

    // non-built-in properties
    let db = database.clone();
    engine.register_indexer_get_result(move |o: &mut O, prop: &str| {
        db.read().get_property_dynamic(o.id, prop)
    });
    let db = database.clone();
    engine.register_indexer_set_result(move |o: &mut O, prop: &str, val: Dynamic| {
        db.write().set_property_dynamic(o.id, prop, val)
    });

    // Other helper functions
    let db = database.clone();
    engine.register_fn("to_string", |o: &mut O| format!("{}", o));

    let db = database;
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

                if values.len() < req_count {
                    bail!(E_INVARG);
                }
                if values.len() > req_count + opt_count && rest.is_none() {
                    bail!(E_INVARG);
                }

                let opt_matched = std::cmp::min(opt_count, values.len() - req_count);
                let mut opt_left = opt_matched;
                let mut val_i = 0;

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
    #[must_use]
    pub fn new(id: ID) -> Self {
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
            bail!(E_INVARG);
        }

        let owner = match value[0].clone().try_cast::<ObjectProxy>() {
            None => bail!(E_INVARG),
            Some(obj) => obj.id,
        };

        let perms: PropertyPerms = value[1].clone().try_into()?;

        let new_name = match value.get(2) {
            None => None,
            Some(d) => {
                if d.is::<String>() {
                    Some(d.clone().as_string()?)
                } else {
                    bail!(E_INVARG)
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
            bail!(E_INVARG);
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
                _ => bail!(E_INVARG),
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
