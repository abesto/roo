use std::collections::{HashMap, HashSet};

use mlua::prelude::*;
use uuid::Uuid;

use crate::command::Command;
use crate::database::verb::VerbSignature;
use crate::database::{Property, Verb};

#[derive(Clone)]
pub struct Object {
    uuid: Uuid,
    pub properties: HashMap<String, Property>,
    pub verbs: HashMap<String, Verb>,
    pub(super) location: Option<Uuid>,
    pub(super) contents: HashSet<Uuid>,
}

impl Object {
    #[must_use]
    pub(crate) fn new(uuid: Uuid) -> Self {
        Object {
            uuid,
            properties: HashMap::new(),
            verbs: HashMap::new(),
            location: None,
            contents: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn location(&self) -> Option<&Uuid> {
        self.location.as_ref()
    }

    #[allow(dead_code)]
    pub fn contents(&self) -> &HashSet<Uuid> {
        &self.contents
    }

    pub fn matching_verb(&self, command: &Command) -> Option<&Verb> {
        let matching_verb = self.verbs.get(command.verb())?;
        match (command, &matching_verb.signature) {
            (Command::VerbNoArgs { verb: _ }, VerbSignature::NoArgs { name: _ }) => {
                Some(matching_verb)
            } // _ => None,
        }
    }
}

impl LuaUserData for Object {}
