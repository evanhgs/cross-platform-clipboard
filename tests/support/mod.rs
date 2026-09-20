use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Base isolée pour les tests : aucune donnée personnelle n'est lue ou modifiée.
pub struct TestDatabase {
    path: PathBuf,
}

impl TestDatabase {
    pub fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self {
            path: std::env::temp_dir().join(format!(
                "cross-platform-clipboard-{label}-{}-{nonce}.sqlite3",
                std::process::id()
            )),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_file(self.path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(self.path.with_extension("sqlite3-shm"));
    }
}
