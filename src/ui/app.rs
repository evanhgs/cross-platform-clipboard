use dioxus::prelude::*;

use crate::application::ClipboardService;
use crate::clipboard::{capture_image, copy_image, copy_text};
use crate::storage::SqliteRepository;

use super::history_panel::HistoryPanel;

static CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    let entries = use_signal(Vec::new);
    let mut status = use_signal(String::new);

    use_future(move || async move {
        loop {
            refresh_history(entries, status);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let visible_entries = entries.read().clone();
    rsx! {
        document::Stylesheet { href: CSS }
        main { class: "clipboard-app",
            header { class: "toolbar",
                div {
                    h1 { "Clipboard" }
                    p { "Local history" }
                }
                div { class: "toolbar-actions",
                    button {
                        class: "secondary",
                        onclick: move |_| refresh_history(entries, status),
                        "Refresh"
                    }
                    button {
                        class: "secondary",
                        onclick: move |_| {
                            match capture_image() {
                                Ok(()) => status.set("Image saved".into()),
                                Err(error) => status.set(format!("Image error: {error}")),
                            }
                            refresh_history(entries, status);
                        },
                        "Save image"
                    }
                    button {
                        class: "danger",
                        onclick: move |_| {
                            match open_service().and_then(|service| service.clear_unpinned()) {
                                Ok(()) => status.set("History cleared".into()),
                                Err(error) => status.set(format!("Clear error: {error}")),
                            }
                            refresh_history(entries, status);
                        },
                        "Clear"
                    }
                }
            }
            if !status.read().is_empty() {
                p { class: "status", "{status}" }
            }
            HistoryPanel {
                entries: visible_entries,
                on_copy: move |text: String| {
                    match copy_text(&text) {
                        Ok(()) => status.set("Text copied".into()),
                        Err(error) => status.set(format!("Copy error: {error}")),
                    }
                },
                on_copy_image: move |path: String| {
                    match copy_image(std::path::Path::new(&path)) {
                        Ok(()) => status.set("Image copied".into()),
                        Err(error) => status.set(format!("Copy error: {error}")),
                    }
                },
                on_delete: move |id: i64| {
                    match open_service().and_then(|service| service.delete_entry(id)) {
                        Ok(()) => status.set("Item deleted".into()),
                        Err(error) => status.set(format!("Delete error: {error}")),
                    }
                    refresh_history(entries, status);
                },
                on_toggle_pin: move |id: i64| {
                    match open_service().and_then(|service| service.toggle_pinned(id)) {
                        Ok(()) => status.set("Pin updated".into()),
                        Err(error) => status.set(format!("Update error: {error}")),
                    }
                    refresh_history(entries, status);
                }
            }
        }
    }
}

fn open_service() -> anyhow::Result<ClipboardService> {
    Ok(ClipboardService::new(SqliteRepository::open()?))
}

fn refresh_history(
    mut entries: Signal<Vec<crate::domain::ClipboardEntry>>,
    mut status: Signal<String>,
) {
    match open_service().and_then(|service| service.recent_entries(200)) {
        Ok(history) if *entries.read() != history => entries.set(history),
        Ok(_) => {}
        Err(error) => status.set(format!("History error: {error}")),
    }
}
