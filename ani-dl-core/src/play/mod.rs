mod mpv;

use anyhow::Result;
use async_trait::async_trait;
use crate::config::Players;

#[async_trait]
pub trait Player {
    async fn play(&self, input: &str) -> Result<()>;
}

pub async fn play(player: &Players, input: &str) -> Result<()> {
    let player = match player {
        Players::Mpv => mpv::Mpv,
    };

    player.play(input).await?;
    Ok(())
}
