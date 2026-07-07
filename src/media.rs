use anyhow::{Result, Context};
use reqwest::Client as ReqwestClient;
use futures::future::join_all;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::process::Stdio;
use std::path::PathBuf;
use kodik_parser::reqwest::Client as KodikParserClient;

use crate::error::{Crash, NotFound};
use crate::types::EpisodeItem;

const EXTERNAL_IDLE_TIMEOUT_SECS: u64 = 60;
const MAX_ATTEMPTS: u32 = 5;

pub async fn play(episode: &EpisodeItem, kodik_parser_client: &KodikParserClient) -> Result<()> {
    let output_path = episode.output_dir.join(&episode.filename);

    if output_path.exists() {
        play_mpv(&output_path.to_string_lossy().to_string()).await?;
    } else {
        let m3u8 = parse_kodik_link(&episode.url, kodik_parser_client).await?;
        play_mpv(&m3u8).await?;
    }

    Ok(())
}

async fn play_mpv(url: &str) -> Result<()> {
    let status = Command::new("mpv")
        .arg("--network-timeout=20")
        .arg(url)
        .status()
        .await
        .context(NotFound::Mpv)?;

    anyhow::ensure!(status.success(), Crash::Mpv);
    Ok(())
}

pub async fn download_selected(
    episodes: &Vec<EpisodeItem>,
    kodik_parser_client: &KodikParserClient,
    reqwest_client: &ReqwestClient,
) -> Result<()> {
    let multi = MultiProgress::new();

    let style = ProgressStyle::with_template(
        "{spinner:.green} {msg} [{bar:30.cyan/blue}] {percent}%"
    )?.progress_chars("⣿⣶⣤⣀");

    let mut tasks = Vec::new();

    for episode in episodes {
        let m3u8 = parse_kodik_link(&episode.url, kodik_parser_client).await?;
        let duration = get_playlist_duration(reqwest_client, &m3u8).await.unwrap_or(0.0);

        let pb = multi.add(ProgressBar::new(duration as u64));
        pb.set_style(style.clone());
        pb.set_message(episode.filename.clone());

        let output_path = episode.output_dir.join(&episode.filename);
        let tmp_path = output_path.with_extension("tmp");

        let task = tokio::spawn(download_with_retry(m3u8, tmp_path, output_path, pb));
        tasks.push(task);
    }

    let results = join_all(tasks).await;
    for result in results {
        result??;
    }
    Ok(())
}

async fn get_playlist_duration(client: &ReqwestClient, m3u8_url: &str) -> Result<f64> {
    let playlist = client.get(m3u8_url).send().await?.text().await?;

    let total: f64 = playlist
        .lines()
        .filter_map(|line| line.strip_prefix("#EXTINF:"))
        .filter_map(|rest| rest.trim_end_matches(',').parse::<f64>().ok())
        .sum();

    Ok(total)
}

async fn download_with_retry(
    url: String,
    tmp_path: PathBuf,
    output_path: PathBuf,
    progress_bar: ProgressBar,
) -> Result<()> {
    let mut last_error = None;

    for attempt in 1..=MAX_ATTEMPTS {
        match download_once(&url, &tmp_path, &progress_bar).await {
            Ok(()) => {
                progress_bar.finish_with_message("Загружено");
                tokio::fs::rename(&tmp_path, &output_path).await?;
                return Ok(());
            }
            Err(e) => {
                progress_bar.set_message(format!(
                    "{} — попытка {attempt}/{MAX_ATTEMPTS}",
                    output_path.file_name().unwrap_or_default().to_string_lossy()
                ));
                last_error = Some(e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    progress_bar.finish_with_message("Ошибка");
    Err(last_error.unwrap())
}

async fn download_once(url: &str, tmp_path: &PathBuf, progress_bar: &ProgressBar) -> Result<()> {
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-headers", "Referer: https://kodikplayer.com/\r\n",
            "-timeout", "30000000",
            "-reconnect", "1",
            "-reconnect_streamed", "1",
            "-reconnect_delay_max", "5",
            "-loglevel", "error",
            "-i", url,
            "-c", "copy",
            "-f", "mp4",
            "-progress", "pipe:1",
        ])
        .arg(tmp_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(NotFound::Ffmpeg)?;

    let ffmpeg_stdout = ffmpeg.stdout.take().unwrap();
    let mut lines = BufReader::new(ffmpeg_stdout).lines();

    loop {
        match timeout(Duration::from_secs(EXTERNAL_IDLE_TIMEOUT_SECS), lines.next_line()).await {
            Ok(Ok(Some(line))) => {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(microseconds) = time_str.parse::<u64>() {
                        progress_bar.set_position(microseconds / 1_000_000);
                    }
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e.into()),
            Err(_elapsed) => {
                ffmpeg.kill().await.ok();
                anyhow::bail!(Crash::Timeout(EXTERNAL_IDLE_TIMEOUT_SECS));
            }
        }
    }

    let status = ffmpeg.wait().await?;

    if !status.success() {
        let mut stderr_output = String::new();
        if let Some(mut stderr) = ffmpeg.stderr.take() {
            stderr.read_to_string(&mut stderr_output).await.ok();
        }
        anyhow::bail!(Crash::Ffmpeg(stderr_output));
    }

    Ok(())
}

async fn parse_kodik_link(url: &str, kodik_parser_client: &KodikParserClient) -> Result<String> {
    let link = format!("https:{url}");
    let kodik_response = kodik_parser::parse(kodik_parser_client, &link).await?;

    let links_720p: Vec<String> = kodik_response.links.quality_720
        .iter()
        .map(|u| u.src.clone())
        .collect();

    Ok(links_720p.first().context(NotFound::Url720p)?.clone())
}
