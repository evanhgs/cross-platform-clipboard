use crate::domain::ClipboardEntry;
use crate::storage::SqliteRepository;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub struct ClipboardService {
    repository: SqliteRepository,
}

impl ClipboardService {
    pub fn new(repository: SqliteRepository) -> Self {
        Self { repository }
    }

    pub fn record_text(&self, text: &str) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            tracing::debug!("ignoring empty clipboard text");
            return Ok(());
        }

        let hash = content_hash(text.as_bytes());
        if self.repository.contains_hash(&hash)? {
            tracing::debug!(text_length = text.len(), content_hash = %hash, "clipboard text already exists");
            return Ok(());
        }
        tracing::debug!(text_length = text.len(), content_hash = %hash, "recording clipboard text");
        self.repository.insert_text(text, &hash)
    }

    pub fn recent_entries(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        tracing::trace!(limit, "loading recent clipboard entries");
        self.repository.list_recent(limit)
    }

    pub fn record_rgba_image(&self, rgba: Vec<u8>, width: u32, height: u32) -> Result<()> {
        let mut hash_input = Vec::with_capacity(rgba.len() + 8);
        hash_input.extend_from_slice(&width.to_le_bytes());
        hash_input.extend_from_slice(&height.to_le_bytes());
        hash_input.extend_from_slice(&rgba);
        let hash = content_hash(&hash_input);
        if self.repository.contains_hash(&hash)? {
            tracing::debug!(width, height, content_hash = %hash, "clipboard image already exists");
            return Ok(());
        }

        let expected_length = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .context("clipboard image dimensions are too large")?;
        anyhow::ensure!(
            rgba.len() == expected_length,
            "clipboard image dimensions do not match its pixels"
        );
        let width = i64::from(width);
        let height = i64::from(height);
        tracing::debug!(
            width,
            height,
            byte_length = rgba.len(),
            "recording clipboard image in SQLite"
        );
        self.repository.insert_image(&rgba, width, height, &hash)
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        tracing::debug!(id, "deleting clipboard entry");
        self.repository.delete_entry(id)
    }

    pub fn toggle_pinned(&self, id: i64) -> Result<()> {
        tracing::debug!(id, "toggling clipboard entry pin");
        self.repository.toggle_pinned(id)
    }

    pub fn clear_unpinned(&self) -> Result<()> {
        tracing::debug!("clearing unpinned clipboard entries");
        self.repository.clear_unpinned()
    }
}

fn content_hash(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

#[cfg(test)]
mod tests {
    use super::content_hash;

    #[test]
    fn hash_is_deterministic_for_the_same_text() {
        assert_eq!(content_hash(b"bonjour"), content_hash(b"bonjour"));
    }

    #[test]
    fn hash_changes_when_text_changes() {
        assert_ne!(content_hash(b"bonjour"), content_hash(b"au revoir"));
    }
}
