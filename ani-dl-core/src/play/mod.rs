mod mpv;

use anyhow::Result;
use async_trait::async_trait;
use crate::types::EpisodeFile;
use crate::config::Players;

#[async_trait]
pub trait Player {
    async fn play(&self, episode: &EpisodeFile, m3u8: &str) -> Result<()>;
}

pub async fn play(player: &Players, episode: &EpisodeFile, m3u8: &str) -> Result<()> {
    let player = match player {
        Players::Mpv => mpv::Mpv,
    };

    player.play(episode, m3u8).await?;
    Ok(())
}
