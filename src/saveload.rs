use std::path::PathBuf;

use crate::database::Database;

#[derive(Clone)]
pub struct SaveloadConfig {
    keep_backups: usize,
    dir: String,
    basename: String,
}

impl SaveloadConfig {
    pub fn current_path(&self) -> PathBuf {
        PathBuf::from(format!("{}/{}.current.ron", self.dir, self.basename))
    }

    fn backup_pattern(&self) -> String {
        return format!("{}/{}.backup.*.ron", self.dir, self.basename);
    }

    fn backup_path(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}.backup.{}.ron",
            self.dir,
            self.basename,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
    }
}

impl Default for SaveloadConfig {
    #[must_use]
    fn default() -> Self {
        SaveloadConfig {
            keep_backups: 10,
            dir: "./database".to_string(),
            basename: "roo".to_string(),
        }
    }
}

pub fn checkpoint(db: &Database, config: &SaveloadConfig) -> Result<String, String> {
    make_backup(config)?;
    let filename = write_checkpoint(db, config)?;
    cleanup_old_backups(config).map_err(|e| e.to_string())?;
    Ok(filename)
}

fn make_backup(config: &SaveloadConfig) -> Result<(), String> {
    let current_path = config.current_path();
    if !current_path.exists() {
        return Ok(());
    }
    std::fs::copy(config.current_path(), config.backup_path())
        .map_err(|e| e.to_string())
        .map(|_| ())
}

fn write_checkpoint(db: &Database, config: &SaveloadConfig) -> Result<String, String> {
    let path = config.current_path();
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    ron::ser::to_writer_pretty(file, db, ron::ser::PrettyConfig::new())
        .map_err(|e| e.to_string())?;
    path.canonicalize()
        .map_err(|e| e.to_string())
        .map(|p| p.to_str().unwrap().to_string())
}

fn cleanup_old_backups(config: &SaveloadConfig) -> std::io::Result<()> {
    let mut files = glob::glob(&config.backup_pattern())
        .unwrap()
        .filter(|path_res| match path_res {
            Ok(path) => path.is_file(),
            _ => false,
        })
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();

    files.sort_unstable_by_key(|x| x.metadata().unwrap().created().unwrap());

    for file in files.iter().rev().skip(config.keep_backups) {
        std::fs::remove_file(file)?;
    }

    Ok(())
}
