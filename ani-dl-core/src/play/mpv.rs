use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::process::Command;
use crate::play::Player;
use crate::types::EpisodeFile;
use crate::error::MpvError;

pub struct Mpv;

#[async_trait]
impl Player for Mpv {
    async fn play(&self, episode: &EpisodeFile, m3u8: &str) -> Result<()> {
        // TODO: Сделать что-то с форматом
        let file = episode.raw_location.with_added_extension("mp4");

        if file.exists() {
            play_mpv(&file.to_string_lossy().to_string()).await?;
        } else {
            play_mpv(&m3u8).await?;
        }

        Ok(())
    }
}

async fn play_mpv(m3u8: &str) -> Result<()> {
    let status = Command::new("mpv")
        .arg("--network-timeout=20")
        .arg(m3u8)
        .status()
        .await
        .context(MpvError::NotFound)?;

    anyhow::ensure!(status.success(), MpvError::Crash);
    Ok(())
}
