use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

const APP_ID: &str = "com.evan.clipboard";
const DESKTOP_FILE: &str = "com.evan.clipboard.desktop";
const LEGACY_DESKTOP_FILE: &str = "cross-platform-clipboard.desktop";

pub fn sync(enabled: bool) -> Result<()> {
    install_launcher()?;
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

fn install_launcher() -> Result<()> {
    let executable = env::current_exe().context("unable to determine the executable path")?;
    let data_home = data_home()?;
    let applications_dir = data_home.join("applications");
    let icons_dir = data_home.join("icons/hicolor/256x256/apps");
    fs::create_dir_all(&applications_dir).with_context(|| {
        format!(
            "unable to create launcher directory {}",
            applications_dir.display()
        )
    })?;
    fs::create_dir_all(&icons_dir)
        .with_context(|| format!("unable to create icon directory {}", icons_dir.display()))?;

    fs::write(
        icons_dir.join(format!("{APP_ID}.png")),
        include_bytes!("../assets/clipboard-logo.png"),
    )
    .context("unable to install the Clipboard icon")?;
    fs::write(
        applications_dir.join(DESKTOP_FILE),
        launcher_entry(&executable),
    )
    .context("unable to install the Clipboard launcher")
}

fn desktop_file_path() -> Result<PathBuf> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".config"),
    };
    Ok(config_home.join("autostart").join(DESKTOP_FILE))
}

fn data_home() -> Result<PathBuf> {
    match env::var_os("XDG_DATA_HOME") {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => {
            Ok(PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".local/share"))
        }
    }
}

fn legacy_desktop_file_path() -> Result<PathBuf> {
    desktop_file_path().map(|path| path.with_file_name(LEGACY_DESKTOP_FILE))
}

fn remove_if_exists(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("unable to remove autostart entry at {}", path.display()))?;
    }
    Ok(())
}

fn desktop_entry(executable: &std::path::Path) -> String {
    // Desktop-entry escaping is intentionally minimal: paths with spaces are quoted.
    let executable = executable.display().to_string().replace('"', "\\\"");
    format!(
        "[Desktop Entry]\nType=Application\nName=Clipboard\nIcon={APP_ID}\nExec=\"{executable}\" --background\nTerminal=false\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

fn launcher_entry(executable: &std::path::Path) -> String {
    let executable = executable.display().to_string().replace('"', "\\\"");
    format!(
        "[Desktop Entry]\nType=Application\nName=Clipboard\nComment=Local clipboard history\nIcon={APP_ID}\nExec=\"{executable}\"\nTerminal=false\nCategories=Utility;\nStartupNotify=true\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autostart_entry_uses_background_mode() {
        assert!(
            desktop_entry(std::path::Path::new("/opt/Clipboard App/clipboard"))
                .contains("--background")
        );
    }

    #[test]
    fn launcher_uses_the_installed_icon() {
        assert!(launcher_entry(std::path::Path::new("/opt/clipboard"))
            .contains("Icon=com.evan.clipboard"));
    }
}
