use anyhow::Result;
use std::hash::{Hash, Hasher};

use crate::storage::SqliteRepository;

/// Orchestration des actions métier, sans dépendre de Dioxus ou de Wayland.
///
/// Étapes pour avancer :
/// 1. Remplacer `hash_text` par SHA-256 (crate `sha2`) avant la persistance.
/// 2. Appeler `record_text` depuis le watcher à chaque nouveau contenu.
/// 3. Ajouter `record_image` après avoir écrit l'image sur disque.
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

        self.repository.insert_text(text, &hash_text(text))
    }
}

fn hash_text(text: &str) -> String {
    // Solution provisoire, stable uniquement dans cette version du binaire.
    // À remplacer par SHA-256 dès l'ajout de la déduplication inter-redémarrage.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
