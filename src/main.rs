use saveload::{RotatingSaveloadConfig, SaveloadConfig};

use crate::database::World;

mod command;
mod database;
mod error;
mod result;
mod saveload;
mod server;

use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "0.42")]
#[clap(setting = AppSettings::ColoredHelp)]
struct CliOpts {
    /// testing | rotating[:dir[:backup_num[:basename]]]
    #[clap(parse(try_from_str = parse_saveload_config))]
    db: SaveloadConfig,
}

fn parse_saveload_config(src: &str) -> Result<SaveloadConfig, String> {
    let parts = src.split(":").collect::<Vec<_>>();
    match parts[0] {
        "testing" => Ok(SaveloadConfig::Testing),
        "rotating" => Ok(SaveloadConfig::Rotating(RotatingSaveloadConfig {
            dir: parts.get(1).unwrap_or(&"./database").to_string(),
            keep_backups: usize::from_str_radix(parts.get(2).unwrap_or(&"10"), 10)
                .map_err(|e| e.to_string())?,
            basename: parts.get(3).unwrap_or(&"roo").to_string(),
        })),
        v => Err(format!("Unsupported DB type: {}", v)),
    }
}

fn main() {
    let opts = CliOpts::parse();
    let saveload_config = opts.db;
    let world = World::from_saveload_config(&saveload_config);
    server::run_server(world, saveload_config.clone());
}
