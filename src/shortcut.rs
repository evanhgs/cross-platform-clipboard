use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use dioxus::desktop::tao::window::Window;

#[cfg(target_os = "linux")]
use std::{
    fs,
    os::unix::net::{UnixListener, UnixStream},
    process::Command,
    thread,
};

static WINDOW: OnceLock<Arc<Window>> = OnceLock::new();
static SOCKET_STARTED: OnceLock<()> = OnceLock::new();

pub const GNOME_SHORTCUT_LABEL: &str = "Ctrl + Alt + C";

pub fn uses_x11_shortcuts() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_SESSION_TYPE").is_ok_and(|session| session.eq_ignore_ascii_case("x11"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

pub fn initialize(window: Arc<Window>) -> Result<()> {
    let _ = WINDOW.set(window);
    start_show_socket()
}

pub fn request_show() -> bool {
    #[cfg(target_os = "linux")]
    {
        UnixStream::connect(show_socket_path())
            .and_then(|mut stream| stream.write_all(b"show"))
            .is_ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

pub fn show_window() {
    if let Some(window) = WINDOW.get() {
        window.set_minimized(false);
        window.set_visible(true);
        window.set_focus();
    }
}

pub fn hide_window() {
    if let Some(window) = WINDOW.get() {
        window.set_visible(false);
    }
}

pub fn sync_gnome_shortcut() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if !std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("gnome")
        {
            return Ok(());
        }
        let path =
            "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/com-evan-clipboard/";
        let schema = "org.gnome.settings-daemon.plugins.media-keys";
        let output = Command::new("gsettings")
            .args(["get", schema, "custom-keybindings"])
            .output()
            .context("unable to inspect GNOME custom shortcuts")?;
        let existing = String::from_utf8_lossy(&output.stdout);
        let list = if existing.contains(path) {
            existing.trim().to_owned()
        } else {
            let entries = existing
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']');
            format!(
                "[{}'{}']",
                if entries.trim().is_empty() {
                    String::new()
                } else {
                    format!("{entries}, ")
                },
                path
            )
        };
        run_gsettings(&["set", schema, "custom-keybindings", &list])?;
        let item_schema =
            format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{path}");
        let executable =
            std::env::current_exe().context("unable to determine executable for GNOME shortcut")?;
        let command = format!(
            "'{}' --show",
            executable.display().to_string().replace('\'', "'\\''")
        );
        run_gsettings(&["set", &item_schema, "name", "Clipboard"])?;
        run_gsettings(&["set", &item_schema, "command", &command])?;
        run_gsettings(&["set", &item_schema, "binding", "<Control><Alt>c"])?;
    }
    Ok(())
}

fn start_show_socket() -> Result<()> {
    #[cfg(target_os = "linux")]
    if SOCKET_STARTED.get().is_none() {
        let path = show_socket_path();
        if UnixStream::connect(&path).is_ok() {
            let _ = SOCKET_STARTED.set(());
            return Ok(());
        }
        if path.exists() {
            fs::remove_file(&path).context("unable to clear stale show socket")?;
        }
        let listener =
            UnixListener::bind(&path).context("unable to start background show listener")?;
        thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut command = [0; 4];
                if stream.read_exact(&mut command).is_ok() && &command == b"show" {
                    show_window();
                }
            }
        });
        let _ = SOCKET_STARTED.set(());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn show_socket_path() -> PathBuf {
    PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap_or_else(|| "/tmp".into()))
        .join("cross-platform-clipboard.show")
}

#[cfg(target_os = "linux")]
fn run_gsettings(args: &[&str]) -> Result<()> {
    let output = Command::new("gsettings")
        .args(args)
        .output()
        .context("unable to run gsettings")?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "GNOME shortcut setup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}
