use rhai::Engine;

use crate::database::SharedDatabase;

pub fn register_api(engine: &mut Engine, database: SharedDatabase) {
    engine.register_fn("echo", |s: &str| s.to_string());

    let db = database.clone();
    engine.register_fn("create", move || db.write().create());

    let db = database.clone();
    engine.register_fn("get_highest_object_number", move || {
        db.read().get_highest_object_number()
    });
}
