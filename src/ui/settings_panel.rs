use dioxus::prelude::*;

#[component]
pub fn SettingsPanel(
    start_at_login: bool,
    on_start_at_login_change: EventHandler<bool>,
    on_save: EventHandler<MouseEvent>,
    on_clear: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        section { class: "settings-page",
            section { class: "settings",
                h3 { "Background launch" }
                p { "Clipboard stays available from the system tray while it runs." }
                label { class: "autostart-toggle",
                    input {
                        r#type: "checkbox",
                        checked: start_at_login,
                        onchange: move |event| on_start_at_login_change.call(event.checked()),
                    }
                    "Start Clipboard in the background when I log in"
                }
                button {
                    class: "secondary",
                    onclick: move |event| on_save.call(event),
                    "Save startup setting"
                }
            }
            section { class: "settings danger-zone",
                h3 { "History" }
                p { "Remove every clipboard item that is not pinned." }
                button {
                    class: "danger",
                    onclick: move |event| on_clear.call(event),
                    "Clear"
                }
            }
        }
    }
}
