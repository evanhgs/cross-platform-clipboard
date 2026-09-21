use cross_platform_clipboard::ui::App;
use cross_platform_clipboard::{autostart, settings::Settings, shortcut};
use std::time::Duration;
use tracing::Level;

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    if std::env::args().any(|arg| arg == "--show") && shortcut::request_show() {
        return;
    }
    cross_platform_clipboard::clipboard::start_text_watcher(Duration::from_millis(750));
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

    let background = std::env::args().any(|arg| arg == "--background");
    let config = dioxus::desktop::Config::new()
        .with_window(dioxus::desktop::WindowBuilder::new().with_visible(!background))
        .with_icon(
            dioxus::desktop::icon_from_memory(include_bytes!("../assets/clipboard-logo.png"))
                .expect("Clipboard logo must be a valid PNG"),
        )
        .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::WindowHides)
        .with_exits_when_last_window_closes(false)
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
