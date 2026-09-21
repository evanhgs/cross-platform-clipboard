use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_start_at_login")]
    pub start_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_at_login: default_start_at_login(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = settings_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("unable to read settings at {}", path.display()))?;
        let settings = toml::from_str::<Self>(&raw)
            .with_context(|| format!("unable to parse settings at {}", path.display()))?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let path = settings_path()?;
        let parent = path.parent().expect("settings path has a parent");
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create settings directory {}", parent.display()))?;
        fs::write(&path, toml::to_string_pretty(self)?)
            .with_context(|| format!("unable to write settings at {}", path.display()))
    }

}

pub fn settings_path() -> Result<PathBuf> {
    ProjectDirs::from("com", "evan", "cross-platform-clipboard")
        .map(|dirs| dirs.config_dir().join("settings.toml"))
        .context("unable to determine the application settings directory")
}

fn default_start_at_login() -> bool {
    true
}
