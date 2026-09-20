use dioxus::prelude::*;

/// État visuel temporaire : à connecter au service quand le watcher texte est prêt.
#[component]
pub fn HistoryPanel() -> Element {
    rsx! {
        section { class: "history-panel",
            h2 { "Historique" }
            p { "Aucun élément enregistré pour le moment." }
        }
    }
}
