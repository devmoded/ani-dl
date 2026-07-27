use clap::Parser;
use std::{collections::HashMap, path::PathBuf};
use anyhow::{Context, Result};
use reqwest::Client;
use futures::future::join_all;
use tokio::sync::mpsc;
use console::style;
use ani_dl_core::shikimori::Shikimori;
use ani_dl_core::types::EpisodeFile;
use ani_dl_core::config::{Config, Mode, APP_NAME, APP_VERSION};
use ani_dl_core::play::play;
use ani_dl_core::engine::{resolve_link, search};
use ani_dl_core::download::{Progress, Status, download, playlist};
use ani_dl_core::error::CliError::EpisodesNotSelected;
use ani_dl_core::error::EngineError::NotFoundSeasons;
use ani_dl_core::error::ConfigError::{KodikApiKeyNotSet, ShikimoriApiUrlNotSet};
use crate::cli::Cli;

pub async fn run_engine() -> Result<()> {
    let cli = Cli::parse();

    const KODIK_KEY_SCRIPT: &str = include_str!("../../scripts/gen_kodik_key.py");
    if cli.gen_kodik_key_script {
        println!("{KODIK_KEY_SCRIPT}");
        return Ok(())
    }

    let config = Config::load()?;
    let mut runtime_config = config.clone();

    if let Some(mode) = cli.mode {
        runtime_config.mode = mode;
    };
    if let Some(quality) = cli.quality {
        runtime_config.quality = quality;
    };
    if let Some(downloader) = cli.downloader {
        runtime_config.downloader = downloader;
    };
    if let Some(engine) = cli.engine {
        runtime_config.engine = engine;
    };
    if let Some(player) = cli.player {
        runtime_config.player = player;
    };

    if cli.verbose {
        println!(
            // TODO: Разобраться с выводом ID и эпизодов
            "Режим: {}; Качество: {}; Рабочий каталог: {}
            \rShikimori ID: {:?}; ID перевода: {:?}
            \rВыбранные эпизоды: {:?}
            \rДвижок: {}; Плеер: {}; Загрузчик: {}",
            style(&runtime_config.mode).green(),
            style(&runtime_config.quality).green(),
            style(&cli.work_dir).green(),
            style(&cli.shikimori_id).green(),
            style(&cli.translate_id).green(),
            style(&cli.episode).green(),
            style(&runtime_config.engine).green(),
            style(&runtime_config.player).green(),
            style(&runtime_config.downloader).green(),
        );
    }

    let kodik_api_key = runtime_config.kodik_api_key;
    anyhow::ensure!(!kodik_api_key.is_empty(), KodikApiKeyNotSet);
    let shikimori_api_url = runtime_config.shikimori_api_url;
    anyhow::ensure!(!shikimori_api_url.is_empty(), ShikimoriApiUrlNotSet);

    let client = Client::builder()
        .user_agent(format!("{APP_NAME}-rust/{APP_VERSION}"))
        .build()?;

    let shikimori_id = match &cli.shikimori_id {
        Some(id) => id.to_string(),
        None => {
            let query = match cli.query {
                Some(query) => query,
                None => inquire::Text::new("Что ищем?").prompt()?,
            };

            let shikimori_response = Shikimori::new(&shikimori_api_url, client.clone())?.search(&query, 10).await?;
            inquire::Select::new("Выберите аниме:", shikimori_response).prompt()?.id.to_string()
        }
    };

    let engine = &runtime_config.engine;

    let search_response = search(
        engine,
        &shikimori_id,
        &kodik_api_key,
    ).await?;

    let translate = match cli.translate_id
        .as_ref()
        .and_then(|id| search_response.releases.iter().find(|r| &r.translation.id == id)) {
            Some(t) => t.clone(),
            None => {
                if let Some(id) = cli.translate_id {
                    println!("{}", style(format!("По указанному ID перевода {id} не удалось найти перевод. Выберите вручную")).red());
                }
                inquire::Select::new("Выберите перевод:", search_response.releases).prompt()?
            }
        };

    let seasons = translate.clone().seasons.unwrap_or_default();

    let season = if seasons.len() > 1 {
        inquire::Select::new("Выберите сезон:", seasons).prompt()?
    } else {
        seasons.first().cloned().context(NotFoundSeasons { engine: engine.clone(), release: translate.clone() })?
    };

    // TODO: Сделать, чтобы путь разворачивал ~ и подобные
    let episodes: Vec<EpisodeFile> = season.episodes
        .iter()
        .map(|ep| EpisodeFile::new(&ep, Some(&translate.to_string()), &PathBuf::from(&cli.work_dir)))
        .collect();

    match runtime_config.mode {
        Mode::Download => {
            let selected_episodes = match &cli.episode {
                Some(nums) => {
                    let (found, not_found): (Vec<&u32>, Vec<&u32>) = nums.iter()
                        .partition(|num| episodes.iter().any(|ep| &ep.num == *num));

                    for num in &not_found {
                        println!("{}", style(format!("Эпизод {num} не найден.")).red());
                    }

                    found.into_iter()
                        .filter_map(|num| episodes.iter().find(|ep| &ep.num == num).cloned())
                        .collect()
                }
                None => Vec::new()
            };

            let selected_episodes = if selected_episodes.is_empty() {
                // TODO: Сделать, чтобы после выбора эпизоды отображались только в виде номеров
                inquire::MultiSelect::new("Выберите эпизоды:", episodes).prompt()?
            } else {
                selected_episodes
            };
            anyhow::ensure!(!selected_episodes.is_empty(), EpisodesNotSelected);


            let (tx, mut rx) = mpsc::channel::<Progress>(100);
            let multi = indicatif::MultiProgress::new();

            let progress_style = indicatif::ProgressStyle::with_template(
                "{spinner:.green} {msg} [{bar:30.cyan/blue}] {percent}%"
            )?.progress_chars("⣿⣶⣤⣀");

            let mut bars: HashMap<String, indicatif::ProgressBar> = HashMap::new();
            let mut tasks = Vec::new();

            for episode in selected_episodes {
                let m3u8 = resolve_link(engine, &episode.raw_link, &runtime_config.quality, &kodik_api_key).await?;
                let m3u8_duration = playlist::m3u8_duration(&client, &m3u8).await.unwrap_or(0.0);

                let pb = multi.add(indicatif::ProgressBar::new(m3u8_duration as u64));
                pb.set_style(progress_style.clone());
                pb.set_message(episode.filename.clone());
                bars.insert(episode.filename.clone(), pb);

                // TODO: Решить проблему с заимствованием, если она есть
                let dl_task = tokio::spawn(download(
                    runtime_config.downloader.clone(),
                    episode, m3u8.clone(), m3u8_duration, tx.clone(),
                ));
                tasks.push(dl_task);
            }

            drop(tx);

            let ui_task = tokio::spawn(async move {
                while let Some(progress) = rx.recv().await {
                    if let Some(pb) = bars.get(&progress.id) {
                        pb.set_position(progress.downloaded_segments);
                        let msg = progress.msg.unwrap_or_default();
                        match progress.status {
                            Status::Downloading => {}
                            Status::Finished => { pb.finish_with_message(style(msg).green().to_string()) }
                            Status::FailAttempt => { pb.set_message(style(msg).yellow().to_string()) }
                            Status::Failed => { pb.set_message(style(msg).red().to_string()) }
                        }
                    }
                }
            });

            let results = join_all(tasks).await;
            for result in results { result?? }
            ui_task.await?;
        }
        Mode::Play => {
            let episode = match cli.episode
                .as_ref()
                .and_then(|nums| nums.first())
                .and_then(|num| episodes.iter().find(|ep| &ep.num == num)) {
                    Some(ep) => ep.clone(),
                    None => {
                        if let Some(num) = cli.episode.as_ref().and_then(|nums| nums.first()) {
                            println!("{}", style(format!("Эпизод {num} не найден. Выберите вручную")).red());
                        }
                        inquire::Select::new("Выберите эпизод:", episodes).prompt()?
                    }
                };

            // TODO: Сделать что-то с форматом
            let file = episode.raw_location.with_added_extension("mp4");

            let player_input = if file.exists() {
                file.to_string_lossy().to_string()
            } else {
                resolve_link(engine, &episode.raw_link, &runtime_config.quality, &kodik_api_key).await?
            };

            play(&runtime_config.player, &player_input).await?;
        }
    }

    Ok(())
}
