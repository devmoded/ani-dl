use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::watch::Sender;
use reqwest::Client;
use crate::download::playlist::Playlist;
use crate::types::EpisodeFile;

pub mod ffmpeg;
pub mod playlist;

pub enum Status {
    Downloading,
    Finished,
    FailAttempt,
    Failed,
}

pub struct Progress {
    pub status: Status,
    pub downloaded_segments: u32,
    pub total_segments: u32,
    pub msg: Option<String>,
    // pub elapsed: Duration,
}

#[async_trait]
pub trait Downloader {
    async fn download_episodes(
        episodes: Vec<EpisodeFile>,
        client: &Client,
        progress_tx: Sender<Progress>
    ) -> Result<()>;
    async fn download(
        episode: EpisodeFile,
        playlist: Playlist,
        progress_tx: Sender<Progress>,
    ) -> Result<()>;
}
