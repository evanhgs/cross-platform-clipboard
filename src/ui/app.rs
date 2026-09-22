use dioxus::prelude::*;

use crate::application::ClipboardService;
use crate::clipboard::{copy_image, copy_text};
use crate::storage::SqliteRepository;
use crate::{autostart, settings::Settings, shortcut};

use super::history_panel::HistoryPanel;
use super::settings_panel::SettingsPanel;

const MAIN_CSS: &str = include_str!("../../assets/main.css");

#[derive(Clone, Copy, PartialEq)]
enum Page {
    History,
    Pinned,
    Settings,
}

#[component]
pub fn App() -> Element {
    let _tray = use_hook(|| {
        use dioxus::desktop::trayicon::menu::{Menu, MenuItem, PredefinedMenuItem};
        let menu = Menu::new();
        let show = MenuItem::with_id("clipboard-show", "Afficher Clipboard", true, None);
        let hide = MenuItem::with_id("clipboard-hide", "Masquer Clipboard", true, None);
        let quit = MenuItem::with_id("clipboard-quit", "Quitter", true, None);
        menu.append_items(&[&show, &hide, &PredefinedMenuItem::separator(), &quit])
            .expect("tray menu must initialize");
        let icon =
            dioxus::desktop::icon_from_memory(include_bytes!("../../assets/clipboard-logo.png"))
                .ok();
        dioxus::desktop::trayicon::init_tray_icon(menu, icon)
    });
    // `tray_icon::menu` is backed by `muda`; Dioxus forwards its menu clicks as
    // `MudaMenuEvent`, not `TrayMenuEvent`. Listening on the generic menu hook
    // makes the actions work with GNOME's AppIndicator implementation.
    dioxus::desktop::use_muda_event_handler(move |event| match event.id().as_ref() {
        "clipboard-show" => shortcut::show_window(),
        "clipboard-hide" => shortcut::hide_window(),
        "clipboard-quit" => std::process::exit(0),
        _ => {}
    });
    let entries = use_signal(Vec::new);
    let status = use_signal(String::new);
    let notification_id = use_signal(|| 0_u64);
    let mut page = use_signal(|| Page::History);
    let initial_settings = Settings::load().unwrap_or_else(|_| Settings::default());
    let mut start_at_login = use_signal(|| initial_settings.start_at_login);

    use_future(move || async move {
        loop {
            refresh_history(entries, status, notification_id);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let all_entries = entries.read().clone();
    let history_entries = all_entries
        .iter()
        .filter(|entry| !entry.pinned)
        .cloned()
        .collect::<Vec<_>>();
    let pinned_entries = all_entries
        .iter()
        .filter(|entry| entry.pinned)
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        document::Style { {MAIN_CSS} }
        main { class: "clipboard-app",
            header { class: "toolbar",

            }
            nav { class: "page-navigation", "aria-label": "Clipboard navigation",
                button {
                    class: if page() == Page::History { "nav-button active" } else { "nav-button" },
                    "aria-current": if page() == Page::History { "page" } else { "false" },
                    onclick: move |_| { tracing::debug!(page = "history", "UI navigation requested"); page.set(Page::History) },
                    "History"
                }
                button {
                    class: if page() == Page::Pinned { "nav-button active" } else { "nav-button" },
                    "aria-current": if page() == Page::Pinned { "page" } else { "false" },
                    onclick: move |_| { tracing::debug!(page = "pinned", "UI navigation requested"); page.set(Page::Pinned) },
                    "Pinned"
                }
                button {
                    class: if page() == Page::Settings { "nav-button active" } else { "nav-button" },
                    "aria-current": if page() == Page::Settings { "page" } else { "false" },
                    onclick: move |_| { tracing::debug!(page = "settings", "UI navigation requested"); page.set(Page::Settings) },
                    "Settings"
                }

            }
            if !status.read().is_empty() {
                aside { class: "toast", role: "status", "aria-live": "polite", "{status}" }
            }
            match page() {
                Page::Settings => rsx! {
                    SettingsPanel {
                        start_at_login: start_at_login(),
                        on_start_at_login_change: move |enabled| start_at_login.set(enabled),
                        on_save: move |_| {
                            tracing::debug!(action = "save_startup_setting", enabled = start_at_login(), "UI action requested");
                            let settings = Settings { start_at_login: start_at_login() };
                            match settings.save().and_then(|_| autostart::sync(settings.start_at_login)) {
                                Ok(()) => show_toast(status, notification_id, if settings.start_at_login { "Autostart enabled" } else { "Autostart disabled" }),
                                Err(error) => {
                                    crate::diagnostics::report_error("save startup setting", &error);
                                    show_toast(status, notification_id, format!("Autostart error: {error}"));
                                }
                            }
                        },
                        on_clear: move |_| {
                            tracing::debug!(action = "clear_history", "UI action requested");
                            match open_service().and_then(|service| service.clear_unpinned()) {
                                Ok(()) => show_toast(status, notification_id, "History cleared"),
                                Err(error) => {
                                    crate::diagnostics::report_error("clear clipboard history", &error);
                                    show_toast(status, notification_id, format!("Clear error: {error}"));
                                }
                            }
                            refresh_history(entries, status, notification_id);
                        },
                    }
                },
                Page::History | Page::Pinned => rsx! {
                    section { class: "history-page",
                        HistoryPanel {
                            entries: if page() == Page::Pinned { pinned_entries } else { history_entries },
                            empty_message: if page() == Page::Pinned { "No pinned items yet." } else { "Just copy with Ctrl + C or Cmd + C" },
                            on_copy: move |text: String| {
                                tracing::debug!(action = "copy_text", text_length = text.len(), "UI action requested");
                                match copy_text(&text) {
                                    Ok(()) => show_toast(status, notification_id, "Text copied"),
                                    Err(error) => {
                                        crate::diagnostics::report_error("copy clipboard text", &error);
                                        show_toast(status, notification_id, format!("Copy error: {error}"));
                                    }
                                }
                            },
                            on_copy_image: move |image: crate::domain::StoredImage| {
                                tracing::debug!(action = "copy_image", width = image.width, height = image.height, "UI action requested");
                                match copy_image(image) {
                                    Ok(()) => show_toast(status, notification_id, "Image copied"),
                                    Err(error) => {
                                        crate::diagnostics::report_error("copy clipboard image", &error);
                                        show_toast(status, notification_id, format!("Copy error: {error}"));
                                    }
                                }
                            },
                            on_delete: move |id: i64| {
                                tracing::debug!(action = "delete_entry", id, "UI action requested");
                                match open_service().and_then(|service| service.delete_entry(id)) {
                                    Ok(()) => show_toast(status, notification_id, "Item deleted"),
                                    Err(error) => {
                                        crate::diagnostics::report_error("delete clipboard entry", &error);
                                        show_toast(status, notification_id, format!("Delete error: {error}"));
                                    }
                                }
                                refresh_history(entries, status, notification_id);
                            },
                            on_toggle_pin: move |id: i64| {
                                tracing::debug!(action = "toggle_pin", id, "UI action requested");
                                match open_service().and_then(|service| service.toggle_pinned(id)) {
                                    Ok(()) => show_toast(status, notification_id, "Pin updated"),
                                    Err(error) => {
                                        crate::diagnostics::report_error("update clipboard entry pin", &error);
                                        show_toast(status, notification_id, format!("Update error: {error}"));
                                    }
                                }
                                refresh_history(entries, status, notification_id);
                            },
                        }
                    }
                },
            }
        }
    }
}

fn open_service() -> anyhow::Result<ClipboardService> {
    Ok(ClipboardService::new(SqliteRepository::open()?))
}

fn refresh_history(
    mut entries: Signal<Vec<crate::domain::ClipboardEntry>>,
    status: Signal<String>,
    notification_id: Signal<u64>,
) {
    match open_service().and_then(|service| service.recent_entries(200)) {
        Ok(history) if *entries.read() != history => entries.set(history),
        Ok(_) => {}
        Err(error) => {
            crate::diagnostics::report_error("refresh clipboard history", &error);
            show_toast(status, notification_id, format!("History error: {error}"));
        }
    }
}

fn show_toast(
    mut status: Signal<String>,
    mut notification_id: Signal<u64>,
    message: impl Into<String>,
) {
    let id = notification_id() + 1;
    notification_id.set(id);
    status.set(message.into());

    spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if notification_id() == id {
            status.set(String::new());
        }
    });
}
