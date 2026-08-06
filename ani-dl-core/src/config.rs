use std::{fs, path::PathBuf};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use strum::{EnumString, Display};
use crate::error::ConfigError;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
// pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_NAME: &str = "ani-dl";

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Play,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Downloaders {
    Ffmpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Engines {
    Kodik,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Players {
    Mpv,
    Cine,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Hd720p,
    Sd480p,
    Low360p,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    pub quality: Quality,
    pub downloader: Downloaders,
    pub engine: Engines,
    pub player: Players,
    pub kodik_api_key: String,
    pub shikimori_api_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Play,
            quality: Quality::Hd720p,
            downloader: Downloaders::Ffmpeg,
            engine: Engines::Kodik,
            player: Players::Mpv,
            kodik_api_key: String::default(),
            shikimori_api_url: "https://shikimori.io/api/animes".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let cfg_path = config_path()?;

        if !cfg_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&cfg_path)
            .context(ConfigError::Read { path: cfg_path.to_string_lossy().to_string() })?;

        let config: Self = toml::from_str(&content)
            .context(ConfigError::Parse { path: cfg_path.to_string_lossy().to_string() })?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let cfg_path = config_path()?;

        if let Some(parent) = cfg_path.parent() {
            fs::create_dir_all(parent)
                .context(ConfigError::CreateDir { path: parent.to_string_lossy().to_string() })?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&cfg_path, contents)
            .context(ConfigError::Write { path: cfg_path.to_string_lossy().to_string() })?;

        Ok(())
    }
}

pub fn config_path() -> Result<PathBuf> {
    // TODO: Решить, менять ли переменную окружения на флаг в CLI
    match std::env::var("ANI_DL_CONFIG").ok() {
        Some(path) => Ok(path.into()),
        None => {
            let cfg_dir = dirs::config_dir().context(ConfigError::NotFound)?.join(APP_NAME);
            Ok(cfg_dir.join("config.toml"))
        }
    }
}
