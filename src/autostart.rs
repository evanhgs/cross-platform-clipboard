use std::env;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::process::Command;

use anyhow::{Context, Result};

#[cfg(target_os = "linux")]
const APP_ID: &str = "com.evan.clipboard";
#[cfg(target_os = "linux")]
const DESKTOP_FILE: &str = "com.evan.clipboard.desktop";
#[cfg(target_os = "linux")]
const LEGACY_DESKTOP_FILE: &str = "cross-platform-clipboard.desktop";

pub fn sync(enabled: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    return sync_windows(enabled);

    #[cfg(target_os = "linux")]
    return sync_linux(enabled);

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = enabled;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn sync_linux(enabled: bool) -> Result<()> {
    remove_if_exists(&user_launcher_file_path()?)?;
    let path = desktop_file_path()?;
    if enabled {
        let parent = path.parent().expect("autostart path has a parent");
        fs::create_dir_all(parent).with_context(|| {
            format!("unable to create autostart directory {}", parent.display())
        })?;
        let executable = env::current_exe().context("unable to determine the executable path")?;
        fs::write(&path, desktop_entry(&executable))
            .with_context(|| format!("unable to write autostart entry at {}", path.display()))?;
    }
    remove_if_exists(&legacy_desktop_file_path()?)?;
    if !enabled {
        remove_if_exists(&path)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn sync_windows(enabled: bool) -> Result<()> {
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "ClipboardHistory";

    if enabled {
        let executable = env::current_exe().context("unable to determine the executable path")?;
        let command = windows_startup_command(&executable);
        run_reg(&[
            "add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &command, "/f",
        ])
    } else {
        let output = Command::new("reg")
            .args(["delete", RUN_KEY, "/v", VALUE_NAME, "/f"])
            .output()
            .context("unable to remove Windows startup entry")?;

        if output.status.success()
            || String::from_utf8_lossy(&output.stderr).contains("Unable to find")
        {
            Ok(())
        } else {
            anyhow::bail!(
                "unable to remove Windows startup entry: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
        }
    }
}

#[cfg(target_os = "windows")]
fn run_reg(args: &[&str]) -> Result<()> {
    let output = Command::new("reg")
        .args(args)
        .output()
        .context("unable to update Windows startup settings")?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "unable to update Windows startup settings: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

#[cfg(target_os = "windows")]
fn windows_startup_command(executable: &std::path::Path) -> String {
    format!("\"{}\" --background", executable.display())
}

#[cfg(target_os = "linux")]
fn desktop_file_path() -> Result<PathBuf> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".config"),
    };
    Ok(config_home.join("autostart").join(DESKTOP_FILE))
}

#[cfg(target_os = "linux")]
fn data_home() -> Result<PathBuf> {
    match env::var_os("XDG_DATA_HOME") {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => {
            Ok(PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".local/share"))
        }
    }
}

#[cfg(target_os = "linux")]
fn user_launcher_file_path() -> Result<PathBuf> {
    Ok(data_home()?.join("applications").join(DESKTOP_FILE))
}

#[cfg(target_os = "linux")]
fn legacy_desktop_file_path() -> Result<PathBuf> {
    desktop_file_path().map(|path| path.with_file_name(LEGACY_DESKTOP_FILE))
}

#[cfg(target_os = "linux")]
fn remove_if_exists(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("unable to remove autostart entry at {}", path.display()))?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn desktop_entry(executable: &std::path::Path) -> String {
    // Desktop-entry escaping is intentionally minimal: paths with spaces are quoted.
    let executable = executable.display().to_string().replace('"', "\\\"");
    format!(
        "[Desktop Entry]\nType=Application\nName=Clipboard\nIcon={APP_ID}\nExec=\"{executable}\" --background\nTerminal=false\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

#[cfg(target_os = "linux")]
#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn autostart_entry_uses_background_mode() {
        assert!(
            desktop_entry(std::path::Path::new("/opt/Clipboard App/clipboard"))
                .contains("--background")
        );
    }
}
