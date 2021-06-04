mod database;
mod server;

fn main() {
    server::run_server().unwrap();
}
