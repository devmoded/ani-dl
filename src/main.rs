mod error;
mod search;
mod cli;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await?;
    Ok(())
}
