use std::collections::HashMap;

use uuid::Uuid;

use crate::{command::Command, database::Object};

use super::{PropertyValue, Verb};

pub struct Database {
    objects: HashMap<Uuid, Object>,
}

impl Database {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn create(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.objects.insert(uuid, Object::new(uuid));
        uuid
    }

    pub fn get(&self, uuid: &Uuid) -> Result<&Object, String> {
        self.objects
            .get(uuid)
            .ok_or_else(|| format!("{} not found", uuid))
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Result<&mut Object, String> {
        self.objects
            .get_mut(uuid)
            .ok_or_else(|| format!("{} not found", uuid))
    }

    pub fn contains_object(&self, uuid: &Uuid) -> bool {
        self.objects.contains_key(uuid)
    }

    pub fn resolve_object(&self, player_uuid: &Uuid, input: &str) -> Option<&Object> {
        let player = self.get(player_uuid).ok()?;
        let candidate_lists = vec![
            // Inventory
            Some(player.contents()),
            // Objects in the location of the player
            player
                .location()
                .and_then(|uuid| self.get(uuid).ok())
                .map_or_else(|| None, |location| Some(location.contents())),
        ];

        let input_uuid = Uuid::parse_str(input).ok();

        for candidate_list_opt in candidate_lists {
            if let Some(candidate_list) = candidate_list_opt {
                for candidate_uuid in candidate_list {
                    if let Ok(candidate) = self.get(candidate_uuid) {
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
        }

        None
    }

    pub fn move_object(&mut self, what_uuid: &Uuid, to_uuid: &Uuid) -> Result<(), String> {
        // Remove from contents of the old location, if any
        if let Some(PropertyValue::Uuid(old_location)) =
            self.get_property(what_uuid, "location")?.cloned()
        {
            self.get_mut(&old_location)?.remove_content(what_uuid);
        }

        // Set new location
        self.get_mut(what_uuid)?
            .set_property("location", Some(*to_uuid));

        // Add to contents of new location
        self.get_mut(to_uuid)?.insert_content(*what_uuid);

        Ok(())
    }

    pub fn chparent(&mut self, uuid_child: &Uuid, uuid_parent: &Uuid) -> Result<(), String> {
        // Remove from old parent, if any
        {
            let opt_uuid_old_parent = self.get(uuid_child)?.parent().cloned();
            if let Some(uuid_old_parent) = opt_uuid_old_parent {
                if let Some(old_parent) = self.objects.get_mut(&uuid_old_parent) {
                    old_parent.remove_child(uuid_child);
                }
            }
        }

        // Set new parent
        {
            let child = self.get_mut(uuid_child)?;
            child.set_property("parent", Some(uuid_parent.clone()));
        }

        // Add child to children of new parent
        {
            let new_parent = self.get_mut(uuid_parent)?;
            new_parent.insert_child(uuid_child.clone());
        }

        Ok(())
    }

    pub fn get_property(&self, uuid: &Uuid, key: &str) -> Result<Option<&PropertyValue>, String> {
        let object = self.get(uuid)?;

        if let Some(value) = object.get_property(key) {
            Ok(Some(value))
        } else if let Some(parent_uuid) = object.parent() {
            self.get_property(parent_uuid, key)
        } else {
            Ok(None)
        }
    }

    pub fn has_verb_with_name(&self, uuid: &Uuid, name: &str) -> Result<bool, String> {
        let object = self.get(uuid)?;

        if object.has_verb_with_name(name) {
            Ok(true)
        } else if let Some(parent_uuid) = object.parent() {
            self.has_verb_with_name(parent_uuid, name)
        } else {
            Ok(false)
        }
    }

    pub fn resolve_verb(&self, uuid: &Uuid, name: &str) -> Result<Option<&Verb>, String> {
        let object = self.get(uuid)?;

        if let Some(verb) = object.resolve_verb(name) {
            Ok(Some(verb))
        } else if let Some(parent_uuid) = object.parent() {
            self.resolve_verb(parent_uuid, name)
        } else {
            Ok(None)
        }
    }

    pub fn matching_verb(&self, uuid: &Uuid, command: &Command) -> Result<Option<&Verb>, String> {
        let object = self.get(uuid)?;

        if let Some(verb) = object.matching_verb(command) {
            Ok(Some(verb))
        } else if let Some(parent_uuid) = object.parent() {
            self.matching_verb(parent_uuid, command)
        } else {
            Ok(None)
        }
    }
}
