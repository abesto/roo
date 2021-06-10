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
    verbs: Vec<Verb>,
}

impl Object {
    #[must_use]
    pub(crate) fn new(uuid: Uuid) -> Self {
        let mut o = Object {
            properties: HashMap::new(),
            verbs: vec![],
        };

        o.properties
            .insert("uuid".to_string(), Property::from(uuid));
        o.properties.insert(
            "location".to_string(),
            Property::from(PropertyValue::UuidOpt(None)),
        );
        o.properties.insert(
            "name".to_string(),
            Property::from(PropertyValue::String(String::new())),
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

    // TODO this needs refactoring

    pub fn uuid(&self) -> &Uuid {
        if let PropertyValue::Uuid(uuid) = &self.properties.get("uuid").unwrap().value {
            uuid
        } else {
            unreachable!(".uuid is always set to a Uuid");
        }
    }

    pub fn name(&self) -> &String {
        if let PropertyValue::String(name) = &self.properties.get("name").unwrap().value {
            name
        } else {
            unreachable!(".name is always set to a String");
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

    pub fn contents(&self) -> &HashSet<Uuid> {
        if let Some(PropertyValue::Uuids(uuids)) = &mut self.get_property("contents") {
            uuids
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
        self.properties
            .insert(key.to_string(), Property::from(value));
    }

    pub fn add_verb(&mut self, verb: Verb) -> Result<(), String> {
        for existing_verb in self.verbs.iter() {
            for existing_name in existing_verb.names() {
                if verb.names().contains(existing_name) {
                    // TODO allow multiple verbs for same name but different arity
                    return Err(format!(
                        "{} already contains verb {}",
                        self.name(),
                        verb.names().join("/")
                    ));
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

    pub fn has_verb_with_name(&self, name: &str) -> bool {
        self.resolve_verb(name).is_some()
    }

    pub fn set_verb_code(&mut self, verb_name: &String, code: String) -> Result<(), String> {
        let verb_opt = self
            .verbs
            .iter_mut()
            .find(|v| v.names().iter().any(|name| name == verb_name));

        if let Some(verb) = verb_opt {
            verb.code = code;
            Ok(())
        } else {
            Err(format!("Verb {} not found on {}", verb_name, self.uuid()))
        }
    }
}

impl LuaUserData for Object {}
