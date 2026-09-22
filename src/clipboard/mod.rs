use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use arboard::Clipboard;

use crate::application::ClipboardService;
use crate::storage::SqliteRepository;

pub fn start_clipboard_watcher(interval: Duration) {
    tracing::debug!(?interval, "starting clipboard watcher");
    thread::Builder::new()
        .name("clipboard-watcher".into())
        .spawn(move || {
            let Ok(repository) = SqliteRepository::open() else {
                log::error!("unable to open clipboard database from watcher");
                return;
            };
            tracing::debug!(database = %repository.database_path().display(), "clipboard watcher database opened");
            let service = ClipboardService::new(repository);
            let Ok(mut clipboard) = Clipboard::new() else {
                log::error!("unable to access the system clipboard");
                return;
            };
            let mut previous = clipboard.get_text().unwrap_or_default();
            tracing::debug!(initial_text_length = previous.len(), "clipboard watcher initialized");

            loop {
                tracing::trace!(?interval, "reading system clipboard");
                match clipboard.get_text() {
                    Ok(text) if text != previous => {
                        tracing::debug!(text_length = text.len(), "clipboard change detected");
                        previous = text.clone();
                        if let Err(error) = service.record_text(&text) {
                            crate::diagnostics::report_error("record clipboard text", &error);
                        }
                    }
                    Ok(_) => tracing::trace!("clipboard is unchanged"),
                    Err(error) => tracing::trace!(error = %error, "clipboard text is unavailable"),
                }
                if let Ok(image) = clipboard.get_image() {
                    match (u32::try_from(image.width), u32::try_from(image.height)) {
                        (Ok(width), Ok(height)) => {
                            if let Err(error) = service.record_rgba_image(image.bytes.into_owned(), width, height) {
                                crate::diagnostics::report_error("record clipboard image", &error);
                            }
                        }
                        _ => log::warn!("clipboard image dimensions are too large to store"),
                    }
                }
                tracing::trace!(?interval, "clipboard watcher sleeping");
                thread::sleep(interval);
            }
        })
        .expect("unable to start clipboard watcher");
}

pub fn copy_text(text: &str) -> Result<()> {
    tracing::debug!(text_length = text.len(), "writing text to system clipboard");
    Clipboard::new()
        .context("unable to access the system clipboard")?
        .set_text(text.to_owned())
        .context("unable to write text to the system clipboard")
}

pub fn copy_image(image: crate::domain::StoredImage) -> Result<()> {
    let width: usize = usize::try_from(image.width).context("saved image width is invalid")?;
    let height: usize = usize::try_from(image.height).context("saved image height is invalid")?;
    let expected_length = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .context("saved image dimensions are too large")?;
    anyhow::ensure!(
        image.rgba.len() == expected_length,
        "saved image pixels are invalid"
    );
    tracing::debug!(
        width,
        height,
        byte_length = image.rgba.len(),
        "writing image to system clipboard"
    );
    Clipboard::new()
        .context("unable to access the system clipboard")?
        .set_image(arboard::ImageData {
            width,
            height,
            bytes: image.rgba.into(),
        })
        .context("unable to write image to the system clipboard")
}
