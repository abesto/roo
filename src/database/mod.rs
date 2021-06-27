#[allow(clippy::clippy::module_inception)]
pub mod database;
pub mod database_proxy;
pub mod object;
pub mod property;
pub mod verb;
pub mod world;

pub use database::Database;
pub use database_proxy::DatabaseProxy;
pub use object::Object;
pub use property::{Property, PropertyValue};
pub use verb::Verb;
pub use world::World;
