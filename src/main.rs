mod error;
mod search;
mod cli;
mod media;
mod types;
mod shikimori;
mod config;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await?;
    Ok(())
}
