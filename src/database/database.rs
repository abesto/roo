use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{command::Command, database::Object, error::Error, error::ErrorCode::*};

use super::{PropertyValue, Verb};

#[derive(Serialize, Deserialize)]
pub struct Database {
    objects: HashMap<Uuid, Object>,
    system_uuid: Uuid,
    players: HashSet<Uuid>,
}

impl Database {
    #[must_use]
    pub(crate) fn new() -> Self {
        let mut db = Self {
            objects: HashMap::new(),
            system_uuid: Uuid::new_v4(), // Fake temporary value
            players: HashSet::new(),
        };

        let system_uuid = db.create_orphan();
        db.system_uuid = system_uuid;

        for name in &["nothing", "failed_match", "ambiguous_match"] {
            let uuid = db.create_orphan();
            {
                let o = db.get_mut_old(&uuid).unwrap();
                o.set_property_old("name", format!("S.{}", name).as_str())
                    .unwrap();
            }
            db.get_mut_old(&system_uuid)
                .unwrap()
                .set_property_old(name, uuid)
                .unwrap();
        }

        db
    }

    pub fn create_orphan(&mut self) -> Uuid {
        let object = Object::new();
        let uuid = *object.uuid();
        self.objects.insert(uuid, object);
        uuid
    }

    pub fn create(&mut self, parent: &Uuid, owner: &Uuid) -> Uuid {
        let mut object = Object::new();
        let uuid = *object.uuid();
        object.set_property_old("parent", Some(*parent)).unwrap();
        object.set_property_old("owner", Some(*owner)).unwrap();
        self.objects.insert(*object.uuid(), object);
        uuid
    }

    pub fn system_uuid(&self) -> &Uuid {
        &self.system_uuid
    }

    pub fn nothing_uuid(&self) -> &Uuid {
        self.get_old(self.system_uuid())
            .unwrap()
            .get_property("nothing")
            .unwrap()
            .try_into()
            .unwrap()
    }

    #[deprecated]
    pub fn get_old(&self, uuid: &Uuid) -> Result<&Object, String> {
        self.objects
            .get(uuid)
            .ok_or_else(|| format!("{} not found", uuid))
    }

    pub fn get(&self, uuid: &Uuid) -> Result<&Object, Error> {
        self.objects
            .get(uuid)
            .ok_or_else(|| E_PERM.make(format!("{} not found", uuid)))
    }

    #[deprecated]
    pub fn get_mut_old(&mut self, uuid: &Uuid) -> Result<&mut Object, String> {
        self.objects
            .get_mut(uuid)
            .ok_or_else(|| format!("{} not found", uuid))
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Result<&mut Object, Error> {
        self.objects
            .get_mut(uuid)
            .ok_or_else(|| E_PERM.make(format!("{} not found", uuid)))
    }

    pub fn contains_object(&self, uuid: &Uuid) -> bool {
        self.objects.contains_key(uuid)
    }

    pub fn resolve_object(&self, player_uuid: &Uuid, input: &str) -> Option<&Object> {
        let player = self.get_old(player_uuid).ok()?;
        let candidate_lists = vec![
            // Inventory
            Some(player.contents()),
            // Objects in the location of the player
            player
                .location()
                .and_then(|uuid| self.get_old(&uuid).ok())
                .map_or_else(|| None, |location| Some(location.contents())),
        ];

        let input_uuid = Uuid::parse_str(input).ok();

        for candidate_list in candidate_lists.into_iter().flatten() {
            for candidate_uuid in candidate_list {
                if let Ok(candidate) = self.get_old(candidate_uuid) {
                    if Some(candidate_uuid) == input_uuid.as_ref() {
                        return Some(candidate);
                    }
                    match candidate.get_property("name") {
                        // TODO add alias support, probably through a method on Object
                        Some(PropertyValue::String(name)) if name == input => {
                            return Some(candidate)
                        }
                        _ => {}
                    }
                }
            }
        }

        None
    }

    pub fn move_object(&mut self, what_uuid: &Uuid, to_uuid: &Uuid) -> Result<(), Error> {
        // Remove from contents of the old location, if any
        if let Some(old_location) = *self.get(what_uuid)?.location() {
            println!("remove_content({}, {})", old_location, what_uuid);
            self.get_mut(&old_location)?.remove_content(what_uuid);
        }

        // Set new location
        self.get_mut(what_uuid)?
            .set_property("location", Some(*to_uuid))?;

        // Add to contents of new location
        self.get_mut(to_uuid)?.insert_content(*what_uuid);

        Ok(())
    }

    pub fn delete(&mut self, what: &Uuid) -> Result<(), String> {
        if let Some(location_uuid) = *self.get_old(what)?.location() {
            if let Ok(location) = self.get_mut_old(&location_uuid) {
                location.remove_content(what);
            }
        }

        if let Some(parent_uuid) = *self.get_old(what)?.parent() {
            if let Ok(parent) = self.get_mut_old(&parent_uuid) {
                parent.remove_child(what);
            }
        }

        self.objects.remove(what);

        Ok(())
    }

    pub fn chparent(&mut self, uuid_child: &Uuid, uuid_parent: &Uuid) -> Result<(), String> {
        // Remove from old parent, if any
        {
            let opt_uuid_old_parent = *self.get_old(uuid_child)?.parent();
            if let Some(uuid_old_parent) = opt_uuid_old_parent {
                if let Some(old_parent) = self.objects.get_mut(&uuid_old_parent) {
                    old_parent.remove_child(uuid_child);
                }
            }
        }

        // Set new parent
        {
            let child = self.get_mut_old(uuid_child)?;
            child.set_property_old("parent", Some(*uuid_parent))?;
        }

        // Add child to children of new parent
        {
            let new_parent = self.get_mut_old(uuid_parent)?;
            new_parent.insert_child(*uuid_child);
        }

        Ok(())
    }

    pub fn get_property(&self, uuid: &Uuid, key: &str) -> Result<Option<&PropertyValue>, String> {
        let object = self.get_old(uuid)?;

        if let Some(value) = object.get_property(key) {
            Ok(Some(value))
        } else if let Some(parent_uuid) = object.parent() {
            self.get_property(parent_uuid, key)
        } else {
            Ok(None)
        }
    }

    pub fn has_verb_with_name(&self, uuid: &Uuid, name: &str) -> Result<bool, String> {
        let object = self.get_old(uuid)?;

        if object.has_verb_with_name(name) {
            Ok(true)
        } else if let Some(parent_uuid) = object.parent() {
            self.has_verb_with_name(parent_uuid, name)
        } else {
            Ok(false)
        }
    }

    pub fn resolve_verb(&self, uuid: &Uuid, name: &str) -> Result<Option<&Verb>, String> {
        let object = self.get_old(uuid)?;

        if let Some(verb) = object.resolve_verb(name) {
            Ok(Some(verb))
        } else if let Some(parent_uuid) = object.parent() {
            self.resolve_verb(parent_uuid, name)
        } else {
            Ok(None)
        }
    }

    pub fn matching_verb(&self, uuid: &Uuid, command: &Command) -> Result<Option<&Verb>, String> {
        let object = self.get_old(uuid)?;

        if let Some(verb) = object.matching_verb(command) {
            Ok(Some(verb))
        } else if let Some(parent_uuid) = object.parent() {
            self.matching_verb(parent_uuid, command)
        } else {
            Ok(None)
        }
    }

    pub fn set_player_flag(&mut self, uuid: &Uuid, val: bool) -> Result<bool, String> {
        self.get_old(uuid)?;
        let old = self.players.contains(uuid);
        if val {
            self.players.insert(*uuid);
        } else {
            self.players.remove(uuid);
        }
        Ok(old)
    }

    pub fn is_player(&self, uuid: &Uuid) -> Result<bool, String> {
        self.get_old(uuid)?;
        Ok(self.players.contains(uuid))
    }

    pub fn players(&self) -> &HashSet<Uuid> {
        &self.players
    }
}
