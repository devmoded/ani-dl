mod error;
mod search;

use anyhow::{Context, Result};
use kodik_api::Client;
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(name = "ani-dl", about = "CLI для просмотра/скачивания аниме")]
struct Cli {
    /// Название аниме-тайтла
    query: Option<String>,

    /// Перейти в режим скачивания
    #[arg(short, long)]
    download: bool,

    /// Путь сохранения аниме (только при скачивании)
    #[arg(short, long, default_value = ".")]
    output: String
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api_key = std::env::var("KODIK_API_KEY").context("")?;
    let client = Client::new(api_key);

    let query = match cli.query {
        Some(value) => value,
        None => inquire::Text::new("Что ищем?").prompt()?,
    };

    let search_response = search::search_titles(&client, &query).await?;

    let releases = search::get_releases(&search_response).await;
    let seasons = search::get_seasons(&releases).await?;
    let episodes = seasons.first().context("adas").map(|(_, s)| search::get_episodes(s).await)?;

    println!("Finded: {}, Filtered: {}", &search_response.total, releases.len());
    // println!("{releases:#?}");
    Ok(())
}
