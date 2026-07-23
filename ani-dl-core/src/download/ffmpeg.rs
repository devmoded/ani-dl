use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use futures::future::join_all;
use tokio::time::{timeout, Duration};
use tokio::{process::Command, sync::watch::Sender};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use std::{process::Stdio, path::PathBuf};
use crate::download::{Downloader, Progress, Status};
use crate::download::playlist::{Playlist, fetch_playlist_info};
use crate::{types::EpisodeFile, error::FfmpegError};

pub struct FfmpegDownloader;

const EXTERNAL_IDLE_TIMEOUT_SECS: u64 = 60;
const MAX_ATTEMPTS: u32 = 5;

#[async_trait]
impl Downloader for FfmpegDownloader {
    async fn download_episodes(
        episodes: Vec<EpisodeFile>,
        client: &Client,
        tx: Sender<Progress>
    ) -> Result<()> {
        let mut tasks = Vec::new();

        for episode in episodes {
            let playlist = fetch_playlist_info(client, &episode.m3u8).await?;
            send_progress(&tx, &playlist, Status::Downloading, 0, None);

            let task = tokio::spawn(Self::download(episode, playlist, tx.clone()));
            tasks.push(task);
        };
        let results = join_all(tasks).await;
        for result in results {
            result??;
        }
        Ok(())
    }
    async fn download(
        episode: EpisodeFile,
        playlist: Playlist,
        tx: Sender<Progress>
    ) -> Result<()> {
        let mut last_error = None;

        let export_format = "mp4";
        let path = episode.raw_location.join(&episode.filename);
        let tmp_path = path.with_extension("tmp");
        let downloaded_path = path.with_extension(&export_format);

        for attempt in 1..=MAX_ATTEMPTS {
            match ffmpeg_download(&tx, &playlist, &export_format, &tmp_path).await {
                Ok(()) => {
                    send_progress(
                        &tx, &playlist, Status::Finished, playlist.total_segments,
                        Some(format!("Загружено: {}", episode.filename))
                    );
                    tokio::fs::rename(&tmp_path, &downloaded_path).await?;
                    return Ok(());
                }
                Err(e) => {
                    send_progress(
                        &tx, &playlist, Status::FailAttempt, playlist.total_segments,
                        Some(format!("{} - попытка {attempt}/{MAX_ATTEMPTS}", episode.filename))
                    );
                    last_error = Some(e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
        let err = last_error.unwrap();
        send_progress(&tx, &playlist, Status::Failed, playlist.total_segments, Some(err.to_string()));
        Err(err)
    }
}

async fn ffmpeg_download(
    tx: &Sender<Progress>,
    playlist: &Playlist,
    export_format: &str,
    export_path: &PathBuf
) -> Result<()> {
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            // TODO: Решить что делать с заголовками
            // "-headers", "Referer: https://kodikplayer.com/\r\n",
            "-timeout", "30000000",
            "-reconnect", "1",
            "-reconnect_streamed", "1",
            "-reconnect_delay_max", "5",
            "-loglevel", "error",
            "-i", &playlist.url,
            "-c", "copy",
            "-f", export_format,
            "-progress", "pipe:1",
        ])
        .arg(export_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(FfmpegError::NotFound)?;

    let ffmpeg_stdout = ffmpeg.stdout.take().unwrap();
    let mut lines = BufReader::new(ffmpeg_stdout).lines();

    loop {
        match timeout(Duration::from_secs(EXTERNAL_IDLE_TIMEOUT_SECS), lines.next_line()).await {
            Ok(Ok(Some(line))) => {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(microseconds) = time_str.parse::<u64>() {
                        send_progress(&tx, playlist, Status::Downloading, (microseconds / 1_000_000) as u32, None);
                    }
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e.into()),
            Err(_elapsed) => {
                ffmpeg.kill().await.ok();
                anyhow::bail!(FfmpegError::Timeout(EXTERNAL_IDLE_TIMEOUT_SECS));
            }
        }
    }

    let status = ffmpeg.wait().await?;

    if !status.success() {
        let mut stderr_output = String::new();
        if let Some(mut stderr) = ffmpeg.stderr.take() {
            stderr.read_to_string(&mut stderr_output).await.ok();
        }
        anyhow::bail!(FfmpegError::Crash { msg: stderr_output });
    }
    Ok(())
}

// TODO: Решить, стоит ли переносить в Progress
fn send_progress(tx: &Sender<Progress>, playlist: &Playlist, status: Status, downloaded: u32, msg: Option<String>) {
    let _ = tx.send(Progress {
        status,
        downloaded_segments: downloaded,
        total_segments: playlist.total_segments,
        msg,
    });
}
