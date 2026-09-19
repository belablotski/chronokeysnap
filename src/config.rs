//! Application configuration: trigger and storage settings, loaded from a TOML file.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub trigger: TriggerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default)]
    pub timer: TimerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub combination: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            combination: "Ctrl+Alt+Shift+S".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendKind {
    Local,
}

impl Default for StorageBackendKind {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub backend: StorageBackendKind,
    #[serde(default)]
    pub local: LocalStorageConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackendKind::default(),
            local: LocalStorageConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub folder: String,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            folder: "~/Pictures/ChronoKeySnap".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trigger: TriggerConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl Config {
    /// Load config from `path`, or fall back to defaults if the file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file {}", path.display()))?;
        Ok(config)
    }

    /// Default config file location: `~/.config/chronokeysnap/config.toml` (platform-specific).
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("could not determine config directory")?;
        Ok(config_dir.join("chronokeysnap").join("config.toml"))
    }

    /// Write the default config to `path` (creating parent dirs), refusing to
    /// overwrite an existing file unless `force` is set.
    pub fn init(path: &Path, force: bool) -> Result<()> {
        if path.exists() && !force {
            anyhow::bail!(
                "config file already exists at {} (use --force to overwrite)",
                path.display()
            );
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        let contents = toml::to_string_pretty(&Self::default())
            .context("failed to serialize default config")?;
        std::fs::write(path, contents)
            .with_context(|| format!("failed to write config file {}", path.display()))?;
        Ok(())
    }

    /// Resolve the local storage folder, expanding a leading `~`.
    pub fn resolved_local_folder(&self) -> PathBuf {
        expand_tilde(&self.storage.local.folder)
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_local_backend() {
        let config = Config::default();
        matches!(config.storage.backend, StorageBackendKind::Local);
        assert_eq!(config.storage.local.folder, "~/Pictures/ChronoKeySnap");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let config = Config::load(Path::new("/nonexistent/path/config.toml")).unwrap();
        assert!(config.trigger.timer.enabled);
        assert_eq!(config.trigger.timer.interval_seconds, 10);
    }

    #[test]
    fn parses_toml_config() {
        let toml_str = r#"
            [trigger.hotkey]
            enabled = true
            combination = "Ctrl+Alt+P"

            [trigger.timer]
            enabled = true
            interval_seconds = 30

            [storage]
            backend = "local"

            [storage.local]
            folder = "/tmp/shots"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.trigger.hotkey.combination, "Ctrl+Alt+P");
        assert_eq!(config.trigger.timer.interval_seconds, 30);
        assert_eq!(config.storage.local.folder, "/tmp/shots");
    }

    #[test]
    fn expands_tilde_in_folder() {
        let mut config = Config::default();
        config.storage.local.folder = "~/shots".to_string();
        let resolved = config.resolved_local_folder();
        assert!(resolved.ends_with("shots"));
        assert!(resolved.is_absolute());
    }
}
