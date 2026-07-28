mod mpv;
mod cine;

use anyhow::Result;
use async_trait::async_trait;
use crate::config::Players;

#[async_trait]
pub trait Player {
    async fn play(&self, input: &str) -> Result<()>;
}

pub async fn play(player: &Players, input: &str) -> Result<()> {
    match player {
        Players::Mpv => mpv::Mpv.play(input).await?,
        Players::Cine => cine::Cine.play(input).await?,
    };

    Ok(())
}
