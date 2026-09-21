use std::borrow::Cow;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use arboard::Clipboard;

use crate::application::ClipboardService;
use crate::storage::SqliteRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardContent {
    Text(String),
    ImagePath(String),
}

pub fn start_text_watcher(interval: Duration) {
    tracing::debug!(?interval, "starting clipboard text watcher");
    thread::Builder::new()
        .name("clipboard-text-watcher".into())
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
                tracing::trace!(?interval, "clipboard watcher sleeping");
                thread::sleep(interval);
            }
        })
        .expect("unable to start clipboard watcher");
}

pub fn capture_image() -> Result<()> {
    tracing::debug!("capturing image from system clipboard");
    let image = Clipboard::new()
        .context("unable to access the system clipboard")?
        .get_image()
        .context("the clipboard does not contain a supported image")?;
    let repository = SqliteRepository::open()?;
    tracing::debug!(
        width = image.width,
        height = image.height,
        byte_length = image.bytes.len(),
        "clipboard image read"
    );
    ClipboardService::new(repository).record_rgba_image(
        image.bytes.into_owned(),
        image.width as u32,
        image.height as u32,
    )
}

pub fn copy_text(text: &str) -> Result<()> {
    tracing::debug!(text_length = text.len(), "writing text to system clipboard");
    Clipboard::new()
        .context("unable to access the system clipboard")?
        .set_text(text.to_owned())
        .context("unable to write text to the system clipboard")
}

pub fn copy_image(path: &std::path::Path) -> Result<()> {
    tracing::debug!(path = %path.display(), "writing image to system clipboard");
    let image = image::open(path)
        .with_context(|| format!("unable to open saved image at {}", path.display()))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Clipboard::new()
        .context("unable to access the system clipboard")?
        .set_image(arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(image.into_raw()),
        })
        .context("unable to write image to the system clipboard")
}
