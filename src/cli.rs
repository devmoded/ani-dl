use anyhow::{Context, Result};
use kodik_api::Client as KodikClient;
use reqwest::Client as ReqwestClient;
use clap::Parser;
use super::error::NotFound;
use super::search;

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
    output: String,

    /// Сгенерировать Python скрипт для получения ключа KODIK API
    #[arg(long)]
    gen_kodik_key_script: bool,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    const KODIK_KEY_SCRIPT: &str = include_str!("../scripts/gen_kodik_key.py");
    if cli.gen_kodik_key_script {
        println!("{KODIK_KEY_SCRIPT}");
        return Ok(())
    }

    let api_key = std::env::var("KODIK_API_KEY").context(NotFound::KodikApiKey)?;
    let kodik_client = KodikClient::new(api_key);

    let query = match cli.query {
        Some(value) => value,
        None => inquire::Text::new("Что ищем?").prompt()?,
    };

    let reqwest_client = ReqwestClient::builder()
        .user_agent("ani-dl-rust/0.1")
        .build()?;

    let shikimori_response = search::search_shikimori(&reqwest_client, &query).await?;
    let selected_anime = inquire::Select::new("Выберите аниме:", shikimori_response).prompt()?;

    let search_response = search::search_titles(&kodik_client, &selected_anime.id.to_string()).await?;

    let releases: Vec<search::ReleaseItem> = search::get_releases(&search_response)
        .await?
        .iter()
        .map(|r| search::ReleaseItem(r))
        .collect();

    let selected_translate = inquire::Select::new("Выберите перевод:", releases).prompt()?;
    let translate = selected_translate.0;

    let seasons = search::get_seasons(translate.clone()).await?;

    let selected_season = if seasons.len() > 1 {
        inquire::Select::new("Выберите сезон:", seasons).prompt()?.1
    } else {
        seasons.first().cloned().context(NotFound::Seasons)?.1
    };

    let episodes = search::get_episodes(&selected_season).await?;
    let selected_episode = inquire::Select::new("Выберите эпизод:", episodes).prompt()?;

    println!("{}", selected_anime);
    println!("{}", selected_translate);
    println!("{}: {}", selected_episode.0, selected_episode.1);
    Ok(())
}
