use std::collections::{HashMap, HashSet};

use mlua::prelude::*;
use uuid::Uuid;

use crate::command::Command;
use crate::database::Property;

use super::PropertyValue;
use super::Verb;

#[derive(Clone)]
pub struct Object {
    properties: HashMap<String, Property>,
    verbs: HashMap<String, String>, // verb name -> property key
}

impl Object {
    #[must_use]
    pub(crate) fn new(uuid: Uuid) -> Self {
        let mut o = Object {
            properties: HashMap::new(),
            verbs: HashMap::new(),
        };

        o.properties
            .insert("uuid".to_string(), Property::from(uuid));
        o.properties.insert(
            "location".to_string(),
            Property::from(PropertyValue::UuidOpt(None)),
        );
        o.properties.insert(
            "contents".to_string(),
            Property::from(HashSet::<Uuid>::new()),
        );
        o.properties.insert(
            "parent".to_string(),
            Property::from(PropertyValue::UuidOpt(None)),
        );
        o.properties.insert(
            "children".to_string(),
            Property::from(HashSet::<Uuid>::new()),
        );

        o
    }

    pub fn uuid(&self) -> &Uuid {
        if let PropertyValue::Uuid(uuid) = &self.properties.get("uuid").unwrap().value {
            uuid
        } else {
            unreachable!(".uuid is always set to a Uuid");
        }
    }

    pub fn location(&self) -> Option<&Uuid> {
        if let Some(PropertyValue::UuidOpt(uuid)) = &self.get_property("location") {
            uuid.as_ref()
        } else {
            unreachable!(".location is always set to an Option<Uuid>")
        }
    }

    pub fn remove_content(&mut self, uuid: &Uuid) {
        if let Some(PropertyValue::Uuids(uuids)) = &mut self.get_property_mut("contents") {
            uuids.remove(uuid);
        } else {
            unreachable!(".contents is always set to a HashSet<Uuid>")
        }
    }

    pub fn insert_content(&mut self, uuid: Uuid) {
        if let Some(PropertyValue::Uuids(uuids)) = &mut self.get_property_mut("contents") {
            uuids.insert(uuid);
        } else {
            unreachable!(".contents is always set to a HashSet<Uuid>")
        }
    }

    pub fn insert_child(&mut self, uuid: Uuid) {
        if let Some(PropertyValue::Uuids(uuids)) = &mut self.get_property_mut("children") {
            uuids.insert(uuid);
        } else {
            unreachable!(".children is always set to a HashSet<Uuid>")
        }
    }

    pub fn remove_child(&mut self, uuid: &Uuid) {
        if let Some(PropertyValue::Uuids(uuids)) = self.get_property_mut("child") {
            uuids.remove(uuid);
        } else {
            unreachable!(".children is always set to a HashSet<Uuid>")
        }
    }

    pub fn parent(&self) -> Option<&Uuid> {
        if let Some(PropertyValue::UuidOpt(uuid)) = &self.get_property("parent") {
            uuid.as_ref()
        } else {
            unreachable!(".parent is always set to an Option<Uuid>")
        }
    }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key).map(|p| &p.value)
    }

    pub fn get_property_mut(&mut self, key: &str) -> Option<&mut PropertyValue> {
        self.properties.get_mut(key).map(|p| &mut p.value)
    }

    pub fn set_property<T>(&mut self, key: &str, from_value: T)
    where
        T: Into<PropertyValue>,
    {
        let value = from_value.into();
        if let PropertyValue::Verb(_verb) = &value {
            // TODO add alias support
            self.verbs.insert(key.to_string(), key.to_string());
        }
        self.properties
            .insert(key.to_string(), Property::from(value));
    }

    pub fn matching_verb(&self, command: &Command) -> Option<&Verb> {
        if let Some(PropertyValue::Verb(matching_verb)) =
            self.get_property(self.verbs.get(command.verb())?)
        {
            if matching_verb.signature.matches(command) {
                Some(matching_verb)
            } else {
                None
            }
        } else {
            unreachable!()
        }
    }

    pub fn contains_verb(&self, verb: &str) -> bool {
        self.verbs.contains_key(verb)
    }
}

impl LuaUserData for Object {}
