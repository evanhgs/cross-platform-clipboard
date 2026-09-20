use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TestDatabase {
    directory: PathBuf,
    path: PathBuf,
}

impl TestDatabase {
    pub fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "cross-platform-clipboard-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        Self {
            path: directory.join("history.sqlite3"),
            directory,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
