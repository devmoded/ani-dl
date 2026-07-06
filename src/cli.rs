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
    gen_key_script: bool,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
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
    let selected_anime = inquire::Select::new("Выберите нужное аниме:", shikimori_response).prompt()?;

    let search_response = search::search_titles(&kodik_client, &selected_anime.id.to_string()).await?;

    let releases: Vec<search::ReleaseItem> = search::get_releases(&search_response)
        .await?
        .iter()
        .map(|r| search::ReleaseItem(r))
        .collect();

    let selected_translate = inquire::Select::new("Выберите перевод:", releases).prompt()?;
    let translate = selected_translate.0;

    // println!("Finded: {}, Filtered: {}", &search_response.total, releases.len());
    println!("{}", selected_anime);
    println!("{}", selected_translate);
    println!("{:#?}", translate);
    // println!("{releases:#?}");
    Ok(())
}
