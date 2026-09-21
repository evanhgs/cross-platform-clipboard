use dioxus::prelude::*;

use crate::domain::{ClipboardEntry, EntryKind};

static PIN_ICON: Asset = asset!("/assets/pin-green.png");
static DELETE_ICON: Asset = asset!("/assets/trash-red.png");

#[component]
pub fn HistoryPanel(
    entries: Vec<ClipboardEntry>,
    empty_message: String,
    on_copy: EventHandler<String>,
    on_copy_image: EventHandler<String>,
    on_delete: EventHandler<i64>,
    on_toggle_pin: EventHandler<i64>,
) -> Element {
    rsx! {
        section { class: "history-panel",
            if entries.is_empty() {
                p { class: "empty", "{empty_message}" }
            }
            for entry in entries {
                article { class: "history-entry", key: "{entry.id}",
                    div { class: "entry-content",
                        div { class: "entry-header",
                            span { class: "entry-kind", "{entry.kind.label()}" }
                            div { class: "entry-actions",
                                button {
                                    class: "icon-button pin-button",
                                    title: if entry.pinned { "Unpin" } else { "Pin" },
                                    "aria-label": if entry.pinned { "Unpin" } else { "Pin" },
                                    onclick: move |_| on_toggle_pin.call(entry.id),
                                    img { src: PIN_ICON, alt: "" }
                                }
                                button {
                                    class: "icon-button danger",
                                    title: "Delete",
                                    "aria-label": "Delete",
                                    onclick: move |_| on_delete.call(entry.id),
                                    img { src: DELETE_ICON, alt: "" }
                                }
                            }
                        }
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
                            }
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

                }
            }
        }
    }
}
