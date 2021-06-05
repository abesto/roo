use std::collections::HashMap;

use uuid::Uuid;

use crate::database::Object;

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

    pub fn get(&self, uuid: &Uuid) -> Option<&Object> {
        self.objects.get(uuid)
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut Object> {
        self.objects.get_mut(uuid)
    }

    pub fn contains_object(&self, uuid: &Uuid) -> bool {
        self.objects.contains_key(uuid)
    }

    // TODO error reporting :)
    pub fn move_object(&mut self, what_uuid: &Uuid, to_uuid: &Uuid) -> Option<()> {
        // Remove from contents of the old location, if any
        if let Some(old_location) = self.objects.get(what_uuid)?.location {
            self.objects
                .get_mut(&old_location)?
                .contents
                .remove(what_uuid);
        }

        // Set new location
        self.objects.get_mut(what_uuid)?.location = Some(*to_uuid);

        // Add to contents of new location
        self.objects.get_mut(to_uuid)?.contents.insert(*what_uuid);

        Some(())
    }
}
