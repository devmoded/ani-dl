mod error;
mod search;
mod cli;
mod media;
mod types;
mod shikimori;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await?;
    Ok(())
}
