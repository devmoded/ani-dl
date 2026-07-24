use ani_dl_cli::engine::run;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}
