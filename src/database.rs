use parking_lot::RwLock;
use rhai::{Array, Dynamic};
use std::{collections::HashMap, sync::Arc};

use crate::error::{Error::*, RhaiResult};

pub type ID = rhai::INT;

#[derive(Debug)]
pub struct Database {
    highest_object_number: ID,
    objects: HashMap<ID, Object>,
}

pub type SharedDatabase = Arc<RwLock<Database>>;

impl Database {
    pub fn new() -> Self {
        let mut objects = HashMap::new();
        objects.insert(0, Object::new(0, -1, -1));
        Self {
            highest_object_number: 0,
            objects,
        }
    }
    pub fn share(self) -> SharedDatabase {
        Arc::new(RwLock::new(self))
    }

    pub fn create(&mut self, parent: ID, owner: ID) -> ID {
        // TODO verify owner
        let id = self.highest_object_number + 1;
        self.highest_object_number = id;
        self.objects.insert(id, Object::new(id, parent, owner));
        id
    }

    pub fn valid(&self, id: ID) -> bool {
        self.objects.contains_key(&id)
    }

    pub fn get_highest_object_number(&self) -> ID {
        self.highest_object_number
    }

    pub fn get_property_dynamic(&self, id: ID, property: &str) -> RhaiResult<Dynamic> {
        if !self.valid(id) {
            bail!(E_INVIND);
        }
        let o = &self.objects[&id];

        if property == "name" {
            Ok(o.name.clone().into())
        } else {
            match o.properties.get(property) {
                None => bail!(E_PROPNF),
                Some(p) => Ok(p.value.clone()),
            }
        }
    }

    pub fn set_property_dynamic(
        &mut self,
        id: ID,
        property: &str,
        value: Dynamic,
    ) -> RhaiResult<()> {
        if !self.valid(id) {
            bail!(E_INVIND);
        }
        let o = self.objects.get_mut(&id).unwrap();
        if property == "name" {
            o.name = value.cast();
            Ok(())
        } else {
            bail!(E_PROPNF)
        }
    }

    pub fn add_property(
        &mut self,
        id: ID,
        name: &str,
        value: Dynamic,
        info: PropertyInfo,
    ) -> RhaiResult<()> {
        if !self.valid(info.owner) || !self.valid(id) {
            bail!(E_INVARG);
        }
        let o = self.objects.get_mut(&id).unwrap();

        // TODO needs to check parent hierarchy
        if o.properties.contains_key(name) {
            bail!(E_INVARG)
        }
        o.properties
            .insert(name.to_string(), Property::new(info, value));
        Ok(())
    }

    pub fn property_info(&self, id: ID, name: &str) -> RhaiResult<&PropertyInfo> {
        if !self.valid(id) {
            bail!(E_INVARG);
        }
        match self.objects[&id].properties.get(name) {
            None => bail!(E_PROPNF),
            Some(p) => Ok(&p.info),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    id: ID,

    // Fundamental Object Attributes
    // https://www.sindome.org/moo-manual.html#fundamental-object-attributes
    is_player: bool,
    parent: ID,
    children: Vec<ID>,

    // Properties on Objects
    // https://www.sindome.org/moo-manual.html#properties-on-objects
    /// the usual name for this object
    name: String,
    /// the player who controls access to the object
    owner: ID,
    /// where the object is in virtual reality
    location: ID,
    /// the inverse of `location`
    contents: Vec<ID>,
    /// does the object have programmer rights?
    programmer: bool,
    /// does the object have wizard rights?
    wizard: bool,
    /// is the object publicly readable?
    r: bool,
    /// is the object publicly writable?
    w: bool,
    /// is the object fertile?
    f: bool,

    /// storage for non-built-in properties
    properties: HashMap<String, Property>,
}

impl Object {
    pub fn new(id: ID, parent: ID, owner: ID) -> Self {
        Self {
            id,
            is_player: false,
            parent,
            children: Vec::new(),
            name: String::new(),
            owner,
            location: -1,
            contents: Vec::new(),
            programmer: false,
            wizard: false,
            r: false,
            w: false,
            f: false,
            properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PropertyPerms {
    pub r: bool,
    pub w: bool,
    pub c: bool,
}

impl PropertyPerms {
    pub fn new(r: bool, w: bool, c: bool) -> Self {
        Self { r, w, c }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub owner: ID,
    pub perms: PropertyPerms,
    pub new_name: Option<String>,
}

impl PropertyInfo {
    pub fn new(owner: ID, perms: PropertyPerms, new_name: Option<String>) -> Self {
        Self {
            owner,
            perms,
            new_name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    info: PropertyInfo,
    value: Dynamic,
}

impl Property {
    fn new(info: PropertyInfo, value: Dynamic) -> Self {
        Self { info, value }
    }
}
