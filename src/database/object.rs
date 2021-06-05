use std::collections::{HashMap, HashSet};

use mlua::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Object {
    uuid: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub(super) location: Option<Uuid>,
    pub(super) contents: HashSet<Uuid>,
}

impl Object {
    #[must_use]
    pub(crate) fn new(uuid: Uuid) -> Self {
        Object {
            uuid,
            name: String::new(),
            properties: HashMap::new(),
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
}

impl LuaUserData for Object {}
