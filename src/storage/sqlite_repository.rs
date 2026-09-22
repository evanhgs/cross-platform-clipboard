use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection};

use crate::domain::ClipboardEntry;

pub struct SqliteRepository {
    connection: Connection,
    database_path: PathBuf,
}

impl SqliteRepository {
    pub fn open() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "github", "cross-platform-clipboard")
            .context("setup directories failed!")?;

        let data_dir = project_dirs.data_local_dir();
        std::fs::create_dir_all(data_dir)?;
        let database_path = data_dir.join("clipboard.sqlite3");
        tracing::debug!(database = %database_path.display(), "opening clipboard SQLite database");
        Self::open_with_path(database_path)
    }
    pub fn open_at(database_path: impl AsRef<Path>) -> Result<Self> {
        let database_path = database_path.as_ref().to_path_buf();
        Self::open_with_path(database_path)
    }

    fn open_with_path(database_path: PathBuf) -> Result<Self> {
        tracing::trace!(database = %database_path.display(), "initializing SQLite repository");
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
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
                image_blob BLOB,
                image_width INTEGER,
                image_height INTEGER,
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
        Self::add_column_if_missing(&conn, "image_blob", "BLOB")?;
        Self::add_column_if_missing(&conn, "image_width", "INTEGER")?;
        Self::add_column_if_missing(&conn, "image_height", "INTEGER")?;
        Self::migrate_legacy_images(&conn)?;

        Ok(Self {
            connection: conn,
            database_path,
        })
    }

    fn add_column_if_missing(connection: &Connection, name: &str, definition: &str) -> Result<()> {
        if !Self::has_column(connection, name)? {
            connection.execute_batch(&format!(
                "ALTER TABLE clipboard_entries ADD COLUMN {name} {definition}"
            ))?;
        }
        Ok(())
    }

    fn has_column(connection: &Connection, name: &str) -> Result<bool> {
        let mut statement = connection.prepare("PRAGMA table_info(clipboard_entries)")?;
        let exists = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .any(|column| column == name);
        Ok(exists)
    }

    fn migrate_legacy_images(connection: &Connection) -> Result<()> {
        if !Self::has_column(connection, "image_path")? {
            return Ok(());
        }
        let legacy_entries = {
            let mut statement = connection.prepare(
                "SELECT id, image_path FROM clipboard_entries
                 WHERE content_type = 'image' AND image_blob IS NULL AND image_path IS NOT NULL",
            )?;
            let entries = statement
                .query_map([], |row| {
                    Ok::<_, rusqlite::Error>((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            entries
        };

        for (id, path) in legacy_entries {
            match image::open(&path) {
                Ok(image) => {
                    let rgba = image.to_rgba8();
                    connection.execute(
                        "UPDATE clipboard_entries
                         SET image_blob = ?1, image_width = ?2, image_height = ?3
                         WHERE id = ?4",
                        params![
                            rgba.as_raw(),
                            i64::from(rgba.width()),
                            i64::from(rgba.height()),
                            id
                        ],
                    )?;
                }
                Err(error) => {
                    tracing::warn!(entry_id = id, path, %error, "unable to migrate legacy clipboard image")
                }
            }
        }
        Ok(())
    }

    pub fn database_path(&self) -> &PathBuf {
        &self.database_path
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

    pub fn insert_image(
        &self,
        rgba: &[u8],
        width: i64,
        height: i64,
        content_hash: &str,
    ) -> Result<()> {
        tracing::trace!(
            byte_length = rgba.len(),
            width,
            height,
            content_hash,
            "inserting image clipboard entry"
        );
        self.connection.execute(
            "INSERT INTO clipboard_entries (content_type, image_blob, image_width, image_height, content_hash) VALUES ('image', ?1, ?2, ?3, ?4)",
            params![rgba, width, height, content_hash],
        )?;
        self.remove_old_unpinned_entries(200)?;
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        tracing::trace!(limit, "querying recent clipboard entries");
        let limit = i64::try_from(limit).context("limit is too large for SQLite")?;
        let mut statement = self.connection.prepare(
            "SELECT id, content_type, text_content, image_blob, image_width, image_height, content_hash, pinned, created_at
             FROM clipboard_entries ORDER BY created_at DESC, id DESC LIMIT ?1",
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

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        tracing::trace!(id, "deleting clipboard entry from SQLite");
        self.connection
            .execute("DELETE FROM clipboard_entries WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn toggle_pinned(&self, id: i64) -> Result<()> {
        tracing::trace!(id, "updating clipboard entry pin in SQLite");
        self.connection.execute(
            "UPDATE clipboard_entries SET pinned = CASE pinned WHEN 0 THEN 1 ELSE 0 END WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    pub fn clear_unpinned(&self) -> Result<()> {
        tracing::trace!("deleting unpinned clipboard entries from SQLite");
        self.connection
            .execute("DELETE FROM clipboard_entries WHERE pinned = 0", [])?;
        Ok(())
    }
}
