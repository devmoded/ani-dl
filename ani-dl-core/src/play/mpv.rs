use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::process::Command;
use crate::play::Player;
use crate::config::Players;
use crate::error::PlayerError;

pub struct Mpv;

#[async_trait]
impl Player for Mpv {
    async fn play(&self, input: &str) -> Result<()> {
        let status = Command::new("mpv")
            .arg("--network-timeout=20")
            .arg(input)
            .status()
            .await
            .context(PlayerError::NotFound { player: Players::Mpv })?;

        anyhow::ensure!(status.success(), PlayerError::Crash { player: Players::Mpv });
        Ok(())
    }
}
