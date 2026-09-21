use dioxus::prelude::*;

#[component]
pub fn SettingsPanel(
    start_at_login: bool,
    on_start_at_login_change: EventHandler<bool>,
    on_save: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        section { class: "settings-page",
            section { class: "settings",
                h3 { "Background shortcut" }
                p { "Ctrl + Alt + C opens Clipboard from anywhere in GNOME." }
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
        }
    }
}
