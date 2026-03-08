use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub default_city: Option<String>,
    pub units: Option<String>,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

fn default_cache_ttl() -> u64 {
    900
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            default_city: None,
            units: None,
            cache_ttl: default_cache_ttl(),
        }
    }
}

impl Config {
    /// Loads config from the TOML file. Returns defaults if the file doesn't
    /// exist or cannot be parsed.
    pub fn load() -> Self {
        let path = Self::config_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    /// Writes the current config to disk, creating parent directories if needed.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Returns the platform-appropriate config file path
    /// (`<config_dir>/mausam/config.toml`).
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mausam")
            .join("config.toml")
    }

    /// Resolves the API key by checking the `MAUSAM_API_KEY` environment
    /// variable first, then falling back to the value stored in the config file.
    pub fn resolve_api_key(&self) -> Option<String> {
        std::env::var("MAUSAM_API_KEY")
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| self.api_key.clone())
    }
}
