use std::{fs::File, path::PathBuf};

use crate::database::Database;

#[derive(Clone)]
pub enum SaveloadConfig {
    Rotating(RotatingSaveloadConfig),
    Testing,
}

impl SaveloadConfig {
    pub fn checkpoint(&self, db: &Database) -> Result<String, String> {
        match self {
            SaveloadConfig::Testing => {
                println!("SaveloadConfig::Testing::checkpoint");
                Ok(String::new())
            }
            SaveloadConfig::Rotating(c) => c.checkpoint(db),
        }
    }

    pub fn load(&self) -> Result<Database, String> {
        match self {
            SaveloadConfig::Testing => {
                println!("SaveloadConfig::Testing::load");
                Err("SaveloadConfig::Testing::load".to_string())
            }
            SaveloadConfig::Rotating(c) => c.load(),
        }
    }
}

#[derive(Clone)]
pub struct RotatingSaveloadConfig {
    pub keep_backups: usize,
    pub dir: String,
    pub basename: String,
}

impl RotatingSaveloadConfig {
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

    pub fn checkpoint(&self, db: &Database) -> Result<String, String> {
        self.make_backup()?;
        let filename = self.write_checkpoint(db)?;
        self.cleanup_old_backups().map_err(|e| e.to_string())?;
        Ok(filename)
    }

    fn make_backup(&self) -> Result<(), String> {
        let current_path = self.current_path();
        if !current_path.exists() {
            return Ok(());
        }
        std::fs::copy(self.current_path(), self.backup_path())
            .map_err(|e| e.to_string())
            .map(|_| ())
    }

    fn write_checkpoint(&self, db: &Database) -> Result<String, String> {
        let path = self.current_path();
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        ron::ser::to_writer_pretty(file, db, ron::ser::PrettyConfig::new())
            .map_err(|e| e.to_string())?;
        path.canonicalize()
            .map_err(|e| e.to_string())
            .map(|p| p.to_str().unwrap().to_string())
    }

    fn cleanup_old_backups(&self) -> std::io::Result<()> {
        let mut files = glob::glob(&self.backup_pattern())
            .unwrap()
            .filter(|path_res| match path_res {
                Ok(path) => path.is_file(),
                _ => false,
            })
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();

        files.sort_unstable_by_key(|x| x.metadata().unwrap().created().unwrap());

        for file in files.iter().rev().skip(self.keep_backups) {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    fn load(&self) -> Result<Database, String> {
        let path = self.current_path();
        if path.exists() {
            println!("Trying to load DB from {:?}", &path);
            let file = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    return Err(format!("Failed to open: {}", e));
                }
            };

            match ron::de::from_reader(file) {
                Ok(db) => Ok(db),
                Err(e) => {
                    return Err(format!("Failed to load DB: {}", e));
                }
            }
        } else {
            return Err(format!("{:?} does not exist", path));
        }
    }
}

impl Default for SaveloadConfig {
    #[must_use]
    fn default() -> Self {
        SaveloadConfig::Rotating(RotatingSaveloadConfig {
            keep_backups: 10,
            dir: "./database".to_string(),
            basename: "roo".to_string(),
        })
    }
}
