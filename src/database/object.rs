use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use mlua::prelude::*;
use paste::paste;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::command::Command;
use crate::database::Property;
use crate::error::{Error, ErrorCode::*};

use super::PropertyValue;
use super::Verb;

macro_rules! getprop {
    ($self:ident, $p:expr, $v:ident, $t:ty) => {
        $self
            .properties
            .get($p)
            .map(|p| {
                assert!(matches!(p.value, PropertyValue::$v { .. }));
                match &p.value {
                    PropertyValue::$v(v) => v,
                    _ => unreachable!(),
                }
            })
            .unwrap()
    };

    ($self:ident, $p:expr, $v:ident) => {
        getprop!($self, $p, $v, _)
    };
}

macro_rules! getprop_mut {
    ($self:ident, $p:expr, $v:ident, $t:ty) => {
        $self
            .properties
            .get_mut($p)
            .map(|p| {
                assert!(matches!(p.value, PropertyValue::$v { .. }));
                match &mut p.value {
                    PropertyValue::$v(v) => v,
                    _ => unreachable!(),
                }
            })
            .unwrap()
    };

    ($self:ident, $p:expr, $v:ident) => {
        getprop_mut!($self, $p, $v, _)
    };
}

macro_rules! prop_getters {
    ($p:ident, $v:ident, $t:ty) => {
        pub fn $p(&self) -> &$t {
            getprop!(self, stringify!($p), $v)
        }

        paste! {
            pub fn[<$p _mut>](&mut self) -> &mut $t {
                getprop_mut!(self, stringify!($p), $v)
            }
        }
    };
}

macro_rules! props {
    ($(($p:ident, $v:ident, $t:ty, $d:expr)),*) => {
        fn is_prop_builtin(&self, key: &str) -> bool {
            $(
                if stringify!($p) == key { return true; }
            )*
            false
        }

        fn load_defaults(&mut self) {
            $(
                self.set_property(stringify!($p), $d).unwrap();
            )*
        }

        $(
            prop_getters!($p, $v, $t);
        )*
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    properties: HashMap<String, Property>,
    verbs: Vec<Verb>,
}

impl Object {
    #[must_use]
    pub(crate) fn new() -> Self {
        let mut o = Object {
            properties: HashMap::new(),
            verbs: vec![],
        };

        o.load_defaults();

        o
    }

    props!(
        (uuid, Uuid, Uuid, Uuid::new_v4()),
        (name, String, String, ""),
        (location, UuidOpt, Option<Uuid>, None),
        (contents, Uuids, HashSet<Uuid>, HashSet::new()),
        (parent, UuidOpt, Option<Uuid>, None),
        (children, Uuids, HashSet<Uuid>, HashSet::new())
    );

    pub fn remove_content(&mut self, uuid: &Uuid) {
        self.contents_mut().remove(uuid);
    }

    pub fn insert_content(&mut self, uuid: Uuid) {
        self.contents_mut().insert(uuid);
    }

    pub fn insert_child(&mut self, uuid: Uuid) {
        self.children_mut().insert(uuid);
    }

    pub fn remove_child(&mut self, uuid: &Uuid) {
        self.children_mut().remove(uuid);
    }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key).map(|p| &p.value)
    }

    pub fn get_property_mut(&mut self, key: &str) -> Option<&mut PropertyValue> {
        self.properties.get_mut(key).map(|p| &mut p.value)
    }

    #[deprecated]
    pub fn set_property_old<T>(
        &mut self,
        key: &str,
        from_value: T,
    ) -> Result<Option<PropertyValue>, String>
    where
        T: Into<PropertyValue>,
    {
        let value = from_value.into();
        let is_builtin = self.is_prop_builtin(key);
        match self.properties.entry(key.to_string()) {
            Entry::Occupied(p) => {
                if key == "uuid" {
                    return Err("UUID is read-only".to_string());
                }
                #[allow(deprecated)]
                p.into_mut().set_old(value, is_builtin).map(Some)
            }
            Entry::Vacant(v) => {
                v.insert(Property::from(value));
                Ok(None)
            }
        }
    }

    pub fn set_property<T>(
        &mut self,
        key: &str,
        from_value: T,
    ) -> Result<Option<PropertyValue>, Error>
    where
        T: Into<PropertyValue>,
    {
        let value = from_value.into();
        let is_builtin = self.is_prop_builtin(key);
        match self.properties.entry(key.to_string()) {
            Entry::Occupied(p) => {
                if key == "uuid" {
                    return Err(E_PERM.make("UUID is read-only"));
                }
                p.into_mut().set(value, is_builtin).map(Some)
            }
            Entry::Vacant(v) => {
                v.insert(Property::from(value));
                Ok(None)
            }
        }
    }

    pub fn set_into_list<T>(
        &mut self,
        key: &str,
        path: Vec<usize>,
        from_value: T,
    ) -> Result<(), String>
    where
        T: Into<PropertyValue>,
    {
        let mut path_so_far = format!("{}.{}", self.uuid(), key);
        let value: PropertyValue = from_value.into();

        let property = match self.get_property_mut(key) {
            Some(p) => p,
            None => return Err(format!("{} has no property {}", self.uuid(), key)),
        };

        let mut this_list = match property {
            PropertyValue::List(l) => l,
            _ => return Err(format!("{}.{} is not a list", self.uuid(), key)),
        };

        for &index in &path[..path.len() - 1] {
            let next_value = match this_list.get_mut(index) {
                Some(v) => v,
                None => return Err(format!("{} has no index {}", path_so_far, index)),
            };

            path_so_far = format!("{}[{}]", path_so_far, index);
            this_list = match next_value {
                PropertyValue::List(l) => l,
                _ => return Err(format!("{} is not a list", path_so_far)),
            };
        }

        let set_index = path[path.len() - 1];
        match set_index.cmp(&this_list.len()) {
            Ordering::Equal => this_list.push(value),
            Ordering::Less => this_list[set_index] = value,
            _ => return Err(format!("{}.len() < {}", path_so_far, set_index)),
        }

        Ok(())
    }

    pub fn add_verb(&mut self, verb: Verb) -> Result<(), Error> {
        for existing_verb in self.verbs.iter() {
            for existing_name in existing_verb.names() {
                if verb.names().contains(existing_name) {
                    // TODO allow multiple verbs for same name but different arity
                    return Err(E_INVARG.make(format!(
                        "{} already contains verb {}",
                        self.name(),
                        verb.names().join("/")
                    )));
                }
            }
        }

        self.verbs.push(verb);
        Ok(())
    }

    pub fn matching_verb(&self, command: &Command) -> Option<&Verb> {
        self.verbs.iter().find(|v| v.matches(&self, command))
    }

    pub fn resolve_verb(&self, name: &str) -> Option<&Verb> {
        self.verbs.iter().find(|v| v.name_matches(name))
    }

    pub fn verb_names(&self) -> Vec<String> {
        self.verbs.iter().map(|v| v.names()[0].clone()).collect()
    }

    pub fn verbs(&self) -> &Vec<Verb> {
        &self.verbs
    }

    pub fn verbs_mut(&mut self) -> &mut Vec<Verb> {
        &mut self.verbs
    }

    pub fn has_verb_with_name(&self, name: &str) -> bool {
        self.resolve_verb(name).is_some()
    }
}

impl LuaUserData for Object {}
