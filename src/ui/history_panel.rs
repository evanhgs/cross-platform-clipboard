use std::io::Cursor;

use base64::Engine;
use dioxus::prelude::*;

use crate::domain::{ClipboardEntry, EntryKind, StoredImage};

#[component]
pub fn HistoryPanel(
    entries: Vec<ClipboardEntry>,
    empty_message: String,
    on_copy: EventHandler<String>,
    on_copy_image: EventHandler<StoredImage>,
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
                                    if entry.pinned { "Unpin" } else { "Pin" }
                                }
                                button {
                                    class: "icon-button danger",
                                    title: "Delete",
                                    "aria-label": "Delete",
                                    onclick: move |_| on_delete.call(entry.id),
                                    "Delete"
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
                            EntryKind::Image => {
                                let image = entry.image.clone();
                                rsx! {
                                    if let Some(image) = image {
                                        button {
                                            class: "entry-image",
                                            title: "Copy image",
                                            onclick: move |_| on_copy_image.call(image.clone()),
                                            if let Some(src) = image_data_url(&image) {
                                                img { class: "entry-preview", src, alt: "Copied image" }
                                            } else {
                                                "Image data unavailable"
                                            }
                                        }
                                    } else {
                                        p { class: "entry-text entry-image", "Image data unavailable" }
                                    }
                                }
                            }
                        }
                    }

                }
            }
        }
    }
}

fn image_data_url(image: &StoredImage) -> Option<String> {
    let width = u32::try_from(image.width).ok()?;
    let height = u32::try_from(image.height).ok()?;
    let rgba = image::RgbaImage::from_raw(width, height, image.rgba.clone())?;
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut png, image::ImageFormat::Png)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png.into_inner())
    ))
}

#[cfg(test)]
mod tests {
    use super::image_data_url;
    use crate::domain::StoredImage;

    #[test]
    fn makes_a_png_data_url_from_stored_rgba() {
        let url = image_data_url(&StoredImage {
            rgba: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
        })
        .unwrap();

        assert!(url.starts_with("data:image/png;base64,"));
    }
}
