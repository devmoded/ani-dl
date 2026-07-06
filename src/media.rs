use anyhow::{Result, Context};
use reqwest::Client as ReqwestClient;
use futures::future::join_all;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::process::Stdio;
use std::path::PathBuf;
use kodik_parser::reqwest::Client;

use crate::error::{Crash, NotFound};
use crate::types::EpisodeItem;

pub async fn play(url: &str) -> Result<()> {
    let status = Command::new("mpv")
        .arg(url)
        .status()
        .await
        .context(NotFound::Mpv)?;

    anyhow::ensure!(status.success(), Crash::Mpv);
    Ok(())
}

pub async fn download_selected(episodes: &Vec<EpisodeItem>, kodik_parser_client: &Client, reqwest_client: &ReqwestClient) -> Result<()> {
    let multi = MultiProgress::new();

    let style = ProgressStyle::with_template(
        "{spinner:.green} {msg} [{bar:30.cyan/blue}] {percent}%"
    )?.progress_chars("⣿⣶⣤⣀");

    let mut tasks = Vec::new();

    for episode in episodes {
        let url = format!("https:{}", episode.url);
        let kodik_response = kodik_parser::parse(kodik_parser_client, &url).await?;
        let link_720p = kodik_response.links.quality_720
            .first()
            .context(NotFound::Url720p)?
            .src
            .clone();

        let m3u8_playlist = reqwest_client
            .get(&link_720p)
            .send()
            .await?
            .text()
            .await?;

        let duration: f64 = m3u8_playlist
            .lines()
            .filter_map(|line| line.strip_prefix("#EXTINF:"))
            .filter_map(|rest| rest.trim_end_matches(',').parse::<f64>().ok())
            .sum();

        let pb = multi.add(ProgressBar::new(duration as u64));
        pb.set_style(style.clone());
        pb.set_message(episode.filename.clone());


        let output_path = episode.output_dir.join(&episode.filename);
        let tmp_path = output_path.with_extension("tmp");
        let task = tokio::spawn(download(link_720p, tmp_path, output_path, pb));
        tasks.push(task);
    }

    let results = join_all(tasks).await;
    for result in results {
        result??;
    }
    return Ok(())
}

async fn download(url: String, tmp_path: PathBuf, output_path: PathBuf, progress_bar: ProgressBar) -> Result<()> {
    let mut ffmpeg = Command::new("ffmpeg")
        .args(["-y", "-i", &url, "-c", "copy", "-f", "mp4", "-progress", "pipe:1"])
        .arg(&tmp_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context(NotFound::Ffmpeg)?;

    let ffmpeg_stdout = ffmpeg.stdout.take().unwrap();
    let mut lines = BufReader::new(ffmpeg_stdout).lines();

    while let Some(line) = lines.next_line().await? {
        if let Some(time_str) = line.strip_prefix("out_time_ms=") {
            if let Ok(microseconds) = time_str.parse::<u64>() {
                progress_bar.set_position(microseconds / 1_000_000);
            }
        }
    }

    let ffmpeg_status = ffmpeg.wait().await?;
    progress_bar.finish_with_message(if ffmpeg_status.success() { "Загружено" } else { "Ошибка" });

    anyhow::ensure!(ffmpeg_status.success(), Crash::Ffmpeg);
    tokio::fs::rename(tmp_path, output_path).await?;
    Ok(())
}
