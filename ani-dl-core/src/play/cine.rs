use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::process::Command;
use crate::play::Player;
use crate::config::Players;
use crate::error::PlayerError;

/// Поддержка плеера [Cine](https://github.com/diegopvlk/Cine)
pub struct Cine;

#[async_trait]
impl Player for Cine {
    async fn play(&self, input: &str) -> Result<()> {
        let status = Command::new("cine")
            .arg(input)
            .status()
            .await
            .context(PlayerError::NotFound { player: Players::Cine })?;

        anyhow::ensure!(status.success(), PlayerError::Crash { player: Players::Cine });
        Ok(())
    }
}
