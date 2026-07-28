mod ffmpeg;
pub mod playlist;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use crate::types::EpisodeFile;
use crate::config::Downloaders;

pub enum Status {
    Downloading,
    Finished,
    FailAttempt,
    Failed,
}

pub struct Progress {
    pub id: String,
    pub status: Status,
    pub downloaded_segments: u64,
    pub total_segments: f64,
    pub msg: Option<String>,
}

#[async_trait]
pub trait Downloader {
    async fn download(
        &self,
        episode: EpisodeFile,
        m3u8: &str,
        duration: f64,
        progress_tx: Sender<Progress>,
    ) -> Result<()>;
}

pub async fn download (
    downloader: Downloaders,
    episode: EpisodeFile,
    m3u8: String,
    duration: f64,
    progress_tx: Sender<Progress>
) -> Result<()> {
    match downloader {
        Downloaders::Ffmpeg => ffmpeg::FfmpegDownloader.download(episode, &m3u8, duration, progress_tx).await?,
    };

    Ok(())
}
