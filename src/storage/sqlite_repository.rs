use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ClipboardEntry;

pub struct SqliteRepository {
    connection: Connection,
    database_path: PathBuf,
    images_dir: PathBuf,
}

impl SqliteRepository {
    pub fn open() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "github", "cross-platform-clipboard")
            .context("setup directories failed!")?;

        let data_dir = project_dirs.data_local_dir();
        fs::create_dir_all(data_dir)?;
        let database_path = data_dir.join("clipboard.sqlite3");
        tracing::debug!(database = %database_path.display(), "opening clipboard SQLite database");
        Self::open_with_paths(database_path, data_dir.join("images"))
    }
    pub fn open_at(database_path: impl AsRef<Path>) -> Result<Self> {
        let database_path = database_path.as_ref().to_path_buf();
        let images_dir = database_path
            .parent()
            .context("database path has no parent directory")?
            .join("images");
        Self::open_with_paths(database_path, images_dir)
    }

    fn open_with_paths(database_path: PathBuf, images_dir: PathBuf) -> Result<Self> {
        tracing::trace!(database = %database_path.display(), images_dir = %images_dir.display(), "initializing SQLite repository");
        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&images_dir)?;

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
            images_dir,
        })
    }

    pub fn database_path(&self) -> &PathBuf {
        &self.database_path
    }

    pub fn images_dir(&self) -> &PathBuf {
        &self.images_dir
    }

    pub fn contains_hash(&self, content_hash: &str) -> Result<bool> {
        tracing::trace!(content_hash, "querying clipboard entry hash");
        let exists: i64 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM clipboard_entries WHERE content_hash = ?1)",
            [content_hash],
            |row| row.get(0),
        )?;
        Ok(exists != 0)
    }

    pub fn insert_text(&self, text: &str, content_hash: &str) -> Result<()> {
        tracing::trace!(
            text_length = text.len(),
            content_hash,
            "inserting text clipboard entry"
        );
        self.connection.execute(
            "INSERT INTO clipboard_entries (content_type, text_content, content_hash) VALUES ('text', ?1, ?2)",
            params![text, content_hash],
        )?;
        self.remove_old_unpinned_entries(200)?;
        Ok(())
    }

    pub fn insert_image(&self, image_path: &Path, content_hash: &str) -> Result<()> {
        tracing::trace!(image_path = %image_path.display(), content_hash, "inserting image clipboard entry");
        self.connection.execute(
            "INSERT INTO clipboard_entries (content_type, image_path, content_hash) VALUES ('image', ?1, ?2)",
            params![image_path.to_string_lossy(), content_hash],
        )?;
        self.remove_old_unpinned_entries(200)?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        tracing::trace!(limit, "querying recent clipboard entries");
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
        tracing::trace!(maximum, "removing expired unpinned clipboard entries");
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

    pub fn delete_entry(&self, id: i64) -> Result<Option<PathBuf>> {
        tracing::trace!(id, "deleting clipboard entry from SQLite");
        let image_path = self
            .connection
            .query_row(
                "SELECT image_path FROM clipboard_entries WHERE id = ?1",
                [id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?;
        self.connection
            .execute("DELETE FROM clipboard_entries WHERE id = ?1", [id])?;
        Ok(image_path.flatten().map(PathBuf::from))
    }

    pub fn toggle_pinned(&self, id: i64) -> Result<()> {
        tracing::trace!(id, "updating clipboard entry pin in SQLite");
        self.connection.execute(
            "UPDATE clipboard_entries SET pinned = CASE pinned WHEN 0 THEN 1 ELSE 0 END WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    pub fn clear_unpinned(&self) -> Result<Vec<PathBuf>> {
        tracing::trace!("deleting unpinned clipboard entries from SQLite");
        let mut statement = self.connection.prepare(
            "SELECT image_path FROM clipboard_entries WHERE pinned = 0 AND image_path IS NOT NULL",
        )?;
        let paths = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(PathBuf::from)
            .collect();
        self.connection
            .execute("DELETE FROM clipboard_entries WHERE pinned = 0", [])?;
        Ok(paths)
    }
}
