use dioxus::prelude::*;

use super::history_panel::HistoryPanel;

static CSS: Asset = asset!("/assets/main.css");

/// Racine Dioxus du MVP.
///
/// Étapes pour avancer :
/// 1. Charger les entrées avec `ClipboardService` au démarrage.
/// 2. Rafraîchir la liste lorsqu'un événement du watcher arrive.
/// 3. Au clic, réécrire le contenu dans le presse-papiers.
#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        main { class: "clipboard-app",
            h1 { "Clipboard" }
            p { "Historique local texte" }
            HistoryPanel {}
        }
    }
}
