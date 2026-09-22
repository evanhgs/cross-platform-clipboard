use rusqlite::Row;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub id: i64,
    pub kind: EntryKind,
    pub text: Option<String>,
    pub image: Option<StoredImage>,
    pub content_hash: String,
    pub pinned: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImage {
    pub rgba: Vec<u8>,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Text,
    Image,
}

impl EntryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Image => "Image",
        }
    }
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
            image: match (row.get(3)?, row.get(4)?, row.get(5)?) {
                (Some(rgba), Some(width), Some(height)) if width > 0 && height > 0 => {
                    Some(StoredImage {
                        rgba,
                        width,
                        height,
                    })
                }
                _ => None,
            },
            content_hash: row.get(6)?,
            pinned: row.get::<_, i64>(7)? != 0,
            created_at: row.get(8)?,
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
                "SELECT 1, 'text', 'bonjour', NULL, NULL, NULL, 'hash', 0, 123",
                [],
                ClipboardEntry::from_row,
            )
            .unwrap();

        assert_eq!(entry.kind, EntryKind::Text);
        assert_eq!(entry.text.as_deref(), Some("bonjour"));
        assert!(!entry.pinned);
    }
}
