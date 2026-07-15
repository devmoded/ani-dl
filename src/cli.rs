use anyhow::{Context, Result};
use kodik_parser::reqwest::Client as KodikParserClient;
use kodik_api::Client as KodikClient;
use reqwest::Client as ReqwestClient;
use std::path::PathBuf;
use clap::Parser;

use crate::media;
use crate::error::{NotFound, ShikimoriError};
use crate::types::ReleaseItem;
use crate::shikimori::Search as ShikimoriSearch;
use crate::search;
use crate::config::{Config, APP_VERSION, APP_NAME};

/// CLI для просмотра/скачивания аниме
///
/// Для указания ключа Kodik API в файле конфигурации
/// (по умолчанию создаётся в `~/.config/ani-dl/config.toml` при первом запуске)
/// напишите `kodik_api_key = "ваш ключ"`
#[derive(Parser, Debug)]
#[command(name = APP_NAME, version)]
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
    let config = Config::load()?;

    const KODIK_KEY_SCRIPT: &str = include_str!("../scripts/gen_kodik_key.py");
    if cli.gen_kodik_key_script {
        println!("{KODIK_KEY_SCRIPT}");
        return Ok(())
    }

    let api_key = config.kodik_api_key.context(NotFound::KodikApiKey)?;
    let kodik_client = KodikClient::new(api_key);

    let query = match cli.query {
        Some(value) => value,
        None => inquire::Text::new("Что ищем?").prompt()?,
    };

    let reqwest_client = ReqwestClient::builder()
        .user_agent(format!("ani-dl-rust/{APP_VERSION}"))
        .build()?;

    let shikimori_response = ShikimoriSearch::new()
        .with_api_url(&config.shikimori_api_url.context(ShikimoriError::ApiUrlNotSet)?)
        .execute(&reqwest_client, &query, 10)
        .await?;
    let selected_anime = inquire::Select::new("Выберите аниме:", shikimori_response).prompt()?;

    let search_response = search::search_titles(&kodik_client, &selected_anime.id.to_string()).await?;

    let releases: Vec<ReleaseItem> = search::get_releases(&search_response)
        .await?
        .iter()
        .map(|r| ReleaseItem(r))
        .collect();

    let selected_translate = inquire::Select::new("Выберите перевод:", releases).prompt()?;
    let translate = selected_translate.0;

    let seasons = search::get_seasons(translate.clone()).await?;

    let selected_season = if seasons.len() > 1 {
        inquire::Select::new("Выберите сезон:", seasons).prompt()?.1
    } else {
        seasons.first().cloned().context(NotFound::Seasons)?.1
    };

    let episodes = search::get_episodes(&selected_season, &selected_translate, &PathBuf::from(&cli.output)).await?;

    let kodik_parser_client = KodikParserClient::new();

    if cli.download {
        let selected_episodes = inquire::MultiSelect::new("Выберите эпизоды:", episodes).prompt()?;
        anyhow::ensure!(!(selected_episodes.len() == 0), NotFound::Select);

        media::download_selected(&selected_episodes, &kodik_parser_client, &reqwest_client).await?;
    } else {
        let selected_episode = inquire::Select::new("Выберите эпизод:", episodes).prompt()?;

        media::play(&selected_episode, &kodik_parser_client).await?;
    }

    Ok(())
}
