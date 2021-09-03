use parking_lot::RwLock;
use rhai::{Dynamic, EvalAltResult};
use std::{collections::HashMap, sync::Arc};

use crate::error::Error;

pub type ID = rhai::INT;

#[derive(Debug, Default)]
pub struct Database {
    highest_object_number: ID,
    objects: HashMap<ID, Object>,
}

pub type SharedDatabase = Arc<RwLock<Database>>;

impl Database {
    pub fn share(self) -> SharedDatabase {
        Arc::new(RwLock::new(self))
    }

    pub fn create(&mut self) -> ID {
        let id = self.highest_object_number + 1;
        self.highest_object_number = id;
        self.objects.insert(id, Object::new(id));
        id
    }

    pub fn valid(&self, id: ID) -> bool {
        self.objects.contains_key(&id)
    }

    pub fn get_highest_object_number(&self) -> ID {
        self.highest_object_number
    }

    pub fn get_property_dynamic(
        &self,
        id: ID,
        property: &str,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        match self.objects.get(&id) {
            None => Err(Error::E_INVIND.into()),
            Some(o) => {
                if property == "name" {
                    Ok(o.name.clone().into())
                } else {
                    Err(Error::E_PROPNF.into())
                }
            }
        }
    }

    pub fn set_property_dynamic(
        &mut self,
        id: ID,
        property: &str,
        value: Dynamic,
    ) -> Result<(), Box<EvalAltResult>> {
        match self.objects.get_mut(&id) {
            None => Err(Error::E_INVIND.into()),
            Some(o) => {
                if property == "name" {
                    o.name = value.cast();
                    Ok(())
                } else {
                    Err(Error::E_PROPNF.into())
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    id: ID,
    name: String,
}

impl Object {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            name: String::new(),
        }
    }
}
