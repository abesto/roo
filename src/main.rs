use saveload::SaveloadConfig;

use crate::database::World;

mod command;
mod database;
mod error;
mod result;
mod saveload;
mod server;

fn main() {
    let saveload_config = SaveloadConfig::default();
    let world = World::from_saveload_config(&saveload_config);
    server::run_server(world, saveload_config.clone());
}
