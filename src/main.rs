use crate::database::World;

mod command;
mod database;
mod server;

fn main() {
    let world = World::new();
    server::run_server(world).unwrap();
}
