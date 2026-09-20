use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection};

use crate::domain::ClipboardEntry;

/// Étapes pour avancer :
/// 1. Ouvrir ce dépôt une seule fois au démarrage dans un worker dédié.
/// 2. Envoyer des commandes au worker (`SaveText`, `ListRecent`, `Delete`) via
///    un canal plutôt que partager `Connection` dans un singleton global.
/// 3. Brancher `ClipboardService::record_text` sur l'observateur clipboard.
/// 4. Ajouter les images comme fichiers dans `images/` : ne stocker ici que
///    leur chemin, leur hash et leurs métadonnées.
pub struct SqliteRepository {
    connection: Connection,
    database_path: PathBuf,
}

impl SqliteRepository {
    pub fn open() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "github", "cross-platform-clipboard")
            .context("setup directories failed!")?;

        let data_dir = project_dirs.data_local_dir();
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(data_dir.join("images"))?;

        Self::open_at(data_dir.join("clipboard.sqlite3"))
    }

    /// Ouvre une base à un emplacement explicite.
    ///
    /// Cette fonction existe aussi pour les tests d'intégration : chaque test
    /// obtient sa propre base temporaire et ne touche jamais à l'historique réel.
    pub fn open_at(database_path: impl AsRef<Path>) -> Result<Self> {
        let database_path = database_path.as_ref().to_path_buf();
        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&database_path)?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA auto_vacuum = INCREMENTAL;

            CREATE TABLE IF NOT EXISTS clipboard_entries(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                text_content TEXT,
                image_path TEXT,
                content_hash TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE INDEX IF NOT EXISTS idx_clipboard_entries_created_at
                ON clipboard_entries(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_clipboard_entries_hash
                ON clipboard_entries(content_hash);
        "#,
        )?;

        Ok(Self {
            connection: conn,
            database_path,
        })
    }

    pub fn database_path(&self) -> &PathBuf {
        &self.database_path
    }

    pub fn insert_text(&self, text: &str, content_hash: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO clipboard_entries (content_type, text_content, content_hash) VALUES ('text', ?1, ?2)",
            params![text, content_hash],
        )?;
        self.remove_old_unpinned_entries(200)?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let limit = i64::try_from(limit).context("limit is too large for SQLite")?;
        let mut statement = self.connection.prepare(
            "SELECT id, content_type, text_content, image_path, content_hash, pinned, created_at
             FROM clipboard_entries ORDER BY pinned DESC, created_at DESC, id DESC LIMIT ?1",
        )?;
        let entries = statement.query_map([limit], ClipboardEntry::from_row)?;
        entries
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn remove_old_unpinned_entries(&self, maximum: usize) -> Result<()> {
        let maximum = i64::try_from(maximum).context("entry limit is too large for SQLite")?;
        self.connection.execute(
            "DELETE FROM clipboard_entries
             WHERE id IN (
                SELECT id FROM clipboard_entries WHERE pinned = 0
                ORDER BY created_at DESC, id DESC LIMIT -1 OFFSET ?1
             )",
            [maximum],
        )?;
        Ok(())
    }
}
