use std::fmt;
use std::path::PathBuf;
use kodik_api::types::{Release, Season};

pub struct ReleaseItem<'a>(pub &'a Release);

impl fmt::Display for ReleaseItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} ({})",
            self.0.title,
            self.0.translation.title,
            self.0.translation.id
        )
    }
}

#[derive(Clone, Debug)]
pub struct SeasonItem(pub u32, pub Season);

impl fmt::Display for SeasonItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Сезон: {}, Эпизодов:{}", self.0, self.1.episodes.len())
    }
}

#[derive(Clone, Debug)]
pub struct EpisodeItem {
    pub num: u32,
    pub url: String,
    pub filename: String,
    pub output_dir: PathBuf,
}

impl EpisodeItem {
    fn is_downloaded(&self) -> bool {
        self.output_dir.join(&self.filename).exists()
    }
}

impl fmt::Display for EpisodeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_downloaded() {
            write!(f, "Эпизод {} [загружено]", self.num)
        } else {
            write!(f, "Эпизод {}", self.num)
        }
    }
}
