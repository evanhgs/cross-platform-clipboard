use dioxus::prelude::*;

use crate::domain::{ClipboardEntry, EntryKind};

#[component]
pub fn HistoryPanel(
    entries: Vec<ClipboardEntry>,
    on_copy: EventHandler<String>,
    on_copy_image: EventHandler<String>,
    on_delete: EventHandler<i64>,
    on_toggle_pin: EventHandler<i64>,
) -> Element {
    rsx! {
        section { class: "history-panel",
            if entries.is_empty() {
                p { class: "empty", "just copy with 'ctrl + c' or 'cmd + c'" }
            }
            for entry in entries {
                article { class: "history-entry", key: "{entry.id}",
                    div { class: "entry-content",
                        span { class: "entry-kind", "{entry.kind.label()}" }
                        match entry.kind {
                            EntryKind::Text => {
                                let text = entry.text.clone().unwrap_or_default();
                                let text_for_copy = text.clone();
                                rsx! {
                                    button {
                                        class: "entry-text",
                                        title: "Copy text",
                                        onclick: move |_| on_copy.call(text_for_copy.clone()),
                                        "{text}"
                                    }
                                }
                            },
                            EntryKind::Image => rsx! {
                                button {
                                    class: "entry-text entry-image",
                                    title: "Copy image",
                                    onclick: {
                                        let path = entry.image_path.clone().unwrap_or_default();
                                        move |_| on_copy_image.call(path.clone())
                                    },
                                    "Saved image"
                                }
                            },
                        }
                    }
                    div { class: "entry-actions",
                        button {
                            class: "icon-button",
                            title: if entry.pinned { "Unpin" } else { "Pin" },
                            onclick: move |_| on_toggle_pin.call(entry.id),
                            if entry.pinned { "Pinned" } else { "Pin" }
                        }
                        button {
                            class: "icon-button danger",
                            title: "Delete",
                            onclick: move |_| on_delete.call(entry.id),
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
