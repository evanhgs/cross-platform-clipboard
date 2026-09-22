#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use cross_platform_clipboard::ui::App;
use cross_platform_clipboard::{autostart, settings::Settings, shortcut};
use std::time::Duration;
use tracing::Level;

fn main() {
    initialize_diagnostics();
    let background = std::env::args().any(|arg| arg == "--background");
    if shortcut::request_show() {
        return;
    }
    cross_platform_clipboard::clipboard::start_clipboard_watcher(Duration::from_millis(750));
    let settings = Settings::load().unwrap_or_else(|error| {
        log::warn!("unable to load settings, using defaults: {error:#}");
        Settings::default()
    });
    if let Err(error) = autostart::sync(settings.start_at_login) {
        log::warn!("unable to update autostart: {error:#}");
    }
    if let Err(error) = shortcut::sync_gnome_shortcut() {
        log::warn!("GNOME shortcut unavailable: {error:#}");
    }

    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Clipboard History")
                .with_visible(!background),
        )
        .with_menu(None)
        .with_disable_context_menu(true)
        .with_icon(
            dioxus::desktop::icon_from_memory(include_bytes!("../assets/clipboard-logo.png"))
                .expect("Clipboard logo must be a valid PNG"),
        )
        .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::WindowCloses)
        .with_exits_when_last_window_closes(true)
        .with_tray_icon_show_window_on_click(true)
        .with_on_window(move |window, _| {
            if let Err(error) = shortcut::initialize(window) {
                log::warn!("background show listener unavailable: {error:#}");
            }
        });
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

fn initialize_diagnostics() {
    dioxus::logger::init(Level::WARN).expect("failed to initialize logger");
}
