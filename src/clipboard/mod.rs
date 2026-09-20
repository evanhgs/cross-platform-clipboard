//! Lecture et surveillance du presse-papiers.
//!
//! Étapes pour avancer :
//! 1. Définir un backend qui lit le texte et, plus tard, l'image courante.
//! 2. Sous Wayland, commencer par un polling non bloquant et comparer le hash
//!    au dernier contenu reçu.
//! 3. Émettre un événement vers `ClipboardService`, jamais une requête SQLite
//!    directement depuis le watcher.
//! 4. Ajouter ensuite les backends natifs par OS si nécessaire.

/// Contenu normalisé avant sa sauvegarde. Les images seront représentées par un
/// chemin local après écriture dans le dossier de données de l'application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    Text(String),
    ImagePath(String),
}
