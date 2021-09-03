use parking_lot::RwLock;
use rhai::Dynamic;
use std::{collections::HashMap, sync::Arc};

use crate::error::{Error, RhaiResult};

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

    pub fn get_property_dynamic(&self, id: ID, property: &str) -> RhaiResult<Dynamic> {
        match self.objects.get(&id) {
            None => bail!(Error::E_INVIND),
            Some(o) => {
                if property == "name" {
                    Ok(o.name.clone().into())
                } else {
                    match o.properties.get(property) {
                        None => bail!(Error::E_PROPNF),
                        Some(p) => Ok(p.value.clone())
                    }
                }
            }
        }
    }

    pub fn set_property_dynamic(
        &mut self,
        id: ID,
        property: &str,
        value: Dynamic,
    ) -> RhaiResult<()> {
        match self.objects.get_mut(&id) {
            None => bail!(Error::E_INVIND),
            Some(o) => {
                if property == "name" {
                    o.name = value.cast();
                    Ok(())
                } else {
                    bail!(Error::E_PROPNF)
                }
            }
        }
    }

    pub fn add_property(
        &mut self,
        id: ID,
        name: &str,
        value: Dynamic,
        info: PropertyInfo,
    ) -> RhaiResult<()> {
        if !self.objects.contains_key(&info.owner) {
            bail!(Error::E_INVARG);
        }
        match self.objects.get_mut(&id) {
            None => bail!(Error::E_INVARG),
            Some(o) => {
                // TODO needs to check parent hierarchy
                if o.properties.contains_key(name) {
                    bail!(Error::E_INVARG)
                }
                o.properties.insert(name.to_string(), Property::new(info, value));
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    id: ID,
    name: String,
    properties: HashMap<String, Property>
}

impl Object {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            name: String::new(),
            properties: HashMap::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PropertyPerms {
    r: bool,
    w: bool,
    c: bool,
}

impl PropertyPerms {
    pub fn new(r: bool, w: bool, c: bool) -> Self {
        Self { r, w, c }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyInfo {
    owner: ID,
    perms: PropertyPerms,
    new_name: Option<String>,
}

impl PropertyInfo {
    pub fn new(owner: ID, perms: PropertyPerms, new_name: Option<String>) -> Self {
        Self { owner, perms, new_name }
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    info: PropertyInfo,
    value: Dynamic
}

impl Property {
    fn new(info: PropertyInfo, value: Dynamic) -> Self {
        Self { info, value}
    }
}