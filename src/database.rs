use parking_lot::RwLock;
use rhai::Dynamic;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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

    fn fertile_or_owner_or_wizard(&self, id: ID) -> bool {
        /*
        // TODO owner? wizard?
        self.objects[&id].f
        */
        true // For now, everyone is a wizard
    }

    pub fn create(&mut self, parent: ID, owner: ID) -> ID {
        // TODO verify owner

        // Either the given parent object must be #-1 or valid and fertile (i.e., its f bit must be set) or else the programmer must own parent or be a wizard; otherwise E_PERM is raised.
        if parent != -1 && (!self.valid(parent) || !self.fertile_or_owner_or_wizard(parent)) {
            return -1;
        }
        let id = self.highest_object_number + 1;
        self.highest_object_number = id;
        self.objects.insert(id, Object::new(id, parent, owner));
        id
    }

    fn is_ancestor(&self, ancestor: ID, descendant: ID) -> bool {
        if ancestor == descendant {
            return true;
        }
        let mut current = descendant;
        while current != -1 {
            if current == ancestor {
                return true;
            }
            current = self.objects[&current].parent;
        }
        false
    }

    fn ancestors_and_self(&self, id: ID) -> Vec<ID> {
        let mut ancestors = Vec::new();
        let mut current = id;
        while current != -1 {
            ancestors.push(current);
            current = self.objects[&current].parent;
        }
        ancestors
    }

    fn descendants_and_self(&self, id: ID) -> Vec<ID> {
        let mut descendants = vec![id];
        for child in self.objects[&id].children.iter() {
            descendants.append(&mut self.descendants_and_self(*child));
        }
        descendants
    }

    pub fn chparent(&mut self, id: ID, parent: ID) -> RhaiResult<()> {
        // If object is not valid, or if new-parent is neither valid nor equal to #-1, then E_INVARG is raised.
        if !self.valid(id) {
            bail!(E_INVARG);
        }
        if !self.valid(parent) && parent != -1 {
            bail!(E_INVARG);
        }

        // If the programmer is neither a wizard or the owner of object, or if new-parent is not fertile
        // (i.e., its f bit is not set) and the programmer is neither the owner of new-parent nor a wizard,
        // then E_PERM is raised.
        if !self.fertile_or_owner_or_wizard(id)
        /* TODO && !owner && !wizard */
        {
            bail!(E_PERM);
        }

        {
            // If new-parent is equal to object or one of its current ancestors, E_RECMOVE is raised.
            if parent == id || self.is_ancestor(id, parent) {
                bail!(E_RECMOVE);
            }

            // If object or one of its descendants defines a property with the same name as one defined
            // either on new-parent or on one of its ancestors, then E_INVARG is raised.
            let ancestor_properties: HashSet<String> = self
                .ancestors_and_self(parent)
                .iter()
                .map(|id| self.objects[id].properties.keys().cloned())
                .flatten()
                .collect();
            for descendant in self.descendants_and_self(id) {
                for property in self.objects[&descendant].properties.keys() {
                    if ancestor_properties.contains(property) {
                        bail!(E_INVARG);
                    }
                }
            }

            // TODO handle adding / removing inherited properties
        }

        let mut object = self.objects.get_mut(&id).unwrap();
        object.parent = parent;
        Ok(())
    }

    pub fn valid(&self, id: ID) -> bool {
        self.objects.contains_key(&id)
    }

    pub fn parent(&self, id: ID) -> ID {
        self.objects
            .get(&id)
            .map(|object| object.parent)
            .unwrap_or(-1)
    }

    pub fn get_highest_object_number(&self) -> ID {
        self.highest_object_number
    }

    pub fn get_name(&self, id: ID) -> RhaiResult<String> {
        if !self.valid(id) {
            bail!(E_INVIND);
        }
        Ok(self.objects[&id].name.clone())
    }

    pub fn set_name(&mut self, id: ID, name: &str) -> RhaiResult<()> {
        if !self.valid(id) {
            bail!(E_INVIND);
        }
        self.objects.get_mut(&id).unwrap().name = name.to_string();
        Ok(())
    }

    pub fn get_property_dynamic(&self, id: ID, property: &str) -> RhaiResult<Dynamic> {
        if !self.valid(id) {
            bail!(E_INVIND);
        }
        let o = &self.objects[&id];
        match o.properties.get(property) {
            None => bail!(E_PROPNF),
            Some(p) => Ok(p.value.clone()),
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
        match o.properties.get_mut(property) {
            None => bail!(E_PROPNF),
            Some(p) => {
                p.value = value;
                Ok(())
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
