use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = ".footnote-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_vault_path: PathBuf,
    pub last_file: Option<String>,
}

impl AppConfig {
    /// Get the config file path (cross-platform)
    pub fn get_config_path() -> Result<PathBuf> {
        let app_dir = crate::platform::get_app_dir()?;
        Ok(app_dir.join(CONFIG_FILENAME))
    }

    /// Load config from disk, returns None if doesn't exist or invalid
    pub fn load() -> Option<AppConfig> {
        let config_path = match Self::get_config_path() {
            Ok(path) => path,
            Err(e) => {
                tracing::warn!("Failed to get config path: {}", e);
                return None;
            }
        };

        if !config_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => Some(config),
                Err(e) => {
                    tracing::warn!("Failed to parse config file: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config file: {}", e);
                None
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let contents = serde_json::to_string_pretty(self)?;

        // Write atomically: write to temp file, then rename
        let temp_path = config_path.with_extension("json.tmp");
        std::fs::write(&temp_path, contents)?;
        std::fs::rename(&temp_path, &config_path)?;

        Ok(())
    }

    /// Delete the config file
    pub fn delete() -> Result<()> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
        }
        Ok(())
    }

    /// Validate that vault exists and is valid
    pub fn validate_vault(&self) -> bool {
        // Check vault path exists
        if !self.last_vault_path.exists() {
            return false;
        }

        // Check .footnotes directory exists
        let footnotes_dir = self.last_vault_path.join(".footnotes");
        if !footnotes_dir.exists() || !footnotes_dir.is_dir() {
            return false;
        }

        true
    }
}
