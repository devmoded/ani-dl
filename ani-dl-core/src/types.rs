use std::{fmt, path::PathBuf};
use anyhow::Result;
use crate::engine::{Engine, Quality};

#[derive(Debug, Clone)]
pub struct Response {
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub title: String,
    pub shikimori_id: Option<String>,
    pub translation: Translation,
    pub seasons: Option<Vec<Season>>,
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.title, self.translation)
    }
}

#[derive(Debug, Clone)]
pub struct Season {
    pub title: Option<String>,
    pub episodes: Vec<Episode>,
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.title {
            Some(title) => write!(f, "Сезон {}: {} эпизодов", title, self.episodes.len()),
            None => write!(f, "{} эпизодов", self.episodes.len())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub num: u32,
    pub link: String,
}

impl Episode {
    pub fn new(num: u32, link: &str) -> Self {
        Self {
            num,
            link: link.to_string(),
        }
    }
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.num, self.link)
    }
}

#[derive(Debug, Clone)]
pub struct EpisodeFile {
    pub m3u8: String,
    pub filename: String,
    pub raw_location: PathBuf,
}

impl EpisodeFile {
    pub async fn new(episode: &Episode, engine: impl Engine, metadata: Option<&str>, output_dir: &PathBuf) -> Result<Self> {
        Ok(Self {
            // Сделать настраиваемое качество
            m3u8: engine.resolve_link(&episode.link, &Quality::Hd720p).await?,
            // TODO: Решить как лучше создавать имя файла
            filename: Self::create_filename(&episode.num, metadata),
            raw_location: output_dir.clone().join(Self::create_filename(&episode.num, metadata)),
        })
    }

    fn create_filename(episode_num: &u32, metadata: Option<&str>) -> String {
        match metadata {
            // TODO: Сделать настраиваемый отступ нулями
            Some(data) => format!("EP{:02} - {}", &episode_num, data),
            None => format!("EP{:02}", &episode_num),
        }
    }
}

impl fmt::Display for EpisodeFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Решить что делать с расширением файла
        if self.raw_location.with_extension("mp4").exists() {
            write!(f, "{} [загружено]", &self.filename)
        } else {
            write!(f, "{}", &self.filename)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub title: String,
    pub id: u32,
}

impl fmt::Display for Translation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.id)
    }
}
