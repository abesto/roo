use std::sync::Arc;
use parking_lot::RwLock;

type ID = u128;

#[derive(Debug, Default)]
pub struct Database {
    highest_object_number: ID
}

impl Database {
    pub fn share(self) -> SharedDatabase {
        Arc::new(RwLock::new(self))
    }

    pub fn create(&mut self) -> ID {
        self.highest_object_number += 1;
        self.highest_object_number
    }

    pub fn get_highest_object_number(&self) -> ID {
        self.highest_object_number
    }
}

pub type SharedDatabase = Arc<RwLock<Database>>;