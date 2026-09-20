use anyhow::{Context, Result};
use std::fs;

use crate::domain::ClipboardEntry;
use crate::storage::SqliteRepository;
use image::RgbaImage;
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
            return Ok(());
        }

        let hash = content_hash(text.as_bytes());
        if self.repository.contains_hash(&hash)? {
            return Ok(());
        }
        self.repository.insert_text(text, &hash)
    }

    pub fn recent_entries(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        self.repository.list_recent(limit)
    }

    pub fn record_rgba_image(&self, rgba: Vec<u8>, width: u32, height: u32) -> Result<()> {
        let mut hash_input = Vec::with_capacity(rgba.len() + 8);
        hash_input.extend_from_slice(&width.to_le_bytes());
        hash_input.extend_from_slice(&height.to_le_bytes());
        hash_input.extend_from_slice(&rgba);
        let hash = content_hash(&hash_input);
        if self.repository.contains_hash(&hash)? {
            return Ok(());
        }

        let image = RgbaImage::from_raw(width, height, rgba)
            .context("clipboard image dimensions do not match its pixels")?;
        let image_path = self.repository.images_dir().join(format!("{hash}.png"));
        image.save(&image_path)?;
        self.repository.insert_image(&image_path, &hash)
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        if let Some(path) = self.repository.delete_entry(id)? {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }

    pub fn toggle_pinned(&self, id: i64) -> Result<()> {
        self.repository.toggle_pinned(id)
    }

    pub fn clear_unpinned(&self) -> Result<()> {
        for path in self.repository.clear_unpinned()? {
            let _ = fs::remove_file(path);
        }
        Ok(())
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
