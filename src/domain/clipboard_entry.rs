use rusqlite::Row;

/// Donnée indépendante de l'interface et du système d'exploitation.
///
/// Étape image : conserver `Image` et `image_path` ; les octets PNG/WebP restent
/// dans le dossier applicatif afin que la base reste petite et administrable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub id: i64,
    pub kind: EntryKind,
    pub text: Option<String>,
    pub image_path: Option<String>,
    pub content_hash: String,
    pub pinned: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Text,
    Image,
}

impl ClipboardEntry {
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let kind = match row.get::<_, String>(1)?.as_str() {
            "text" => EntryKind::Text,
            "image" => EntryKind::Image,
            _ => {
                return Err(rusqlite::Error::InvalidColumnType(
                    1,
                    "content_type".into(),
                    rusqlite::types::Type::Text,
                ))
            }
        };

        Ok(Self {
            id: row.get(0)?,
            kind,
            text: row.get(2)?,
            image_path: row.get(3)?,
            content_hash: row.get(4)?,
            pinned: row.get::<_, i64>(5)? != 0,
            created_at: row.get(6)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{ClipboardEntry, EntryKind};

    #[test]
    fn parses_a_text_entry_from_sqlite() {
        let connection = Connection::open_in_memory().unwrap();
        let entry = connection
            .query_row(
                "SELECT 1, 'text', 'bonjour', NULL, 'hash', 0, 123",
                [],
                ClipboardEntry::from_row,
            )
            .unwrap();

        assert_eq!(entry.kind, EntryKind::Text);
        assert_eq!(entry.text.as_deref(), Some("bonjour"));
        assert!(!entry.pinned);
    }
}
