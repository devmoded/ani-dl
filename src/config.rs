use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub kodik_api_key: Option<String>,
    pub shikimori_api_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kodik_api_key: None,
            shikimori_api_url: Some("https://shikimori.io/api/animes".to_string()),
        }
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        match std::env::var("ANI_DL_CONFIG").ok() {
            Some(value) => Ok(value.into()),
            None => {
                let dir = dirs::config_dir().context(ConfigError::NotFound)?.join("ani-dl");
                Ok(dir.join("config.toml"))
            }
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&path)
            .context(ConfigError::Read(path.to_string_lossy().to_string()))?;

        let config: Self = toml::from_str(&contents)
            .context(ConfigError::Parse(path.to_string_lossy().to_string()))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context(ConfigError::CreateDir(parent.to_string_lossy().to_string()))?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)
            .context(ConfigError::Write(path.to_string_lossy().to_string()))?;

        Ok(())
    }
}
