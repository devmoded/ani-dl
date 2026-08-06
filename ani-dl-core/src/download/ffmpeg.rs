use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::time::{timeout, Duration};
use tokio::{process::Command, sync::mpsc::Sender};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use std::{process::Stdio, path::PathBuf};
use crate::download::{Downloader, Progress, Status};
use crate::config::Downloaders;
use crate::{types::EpisodeFile, error::DownloaderError};

const EXTERNAL_IDLE_TIMEOUT_SECS: u64 = 60;
const MAX_ATTEMPTS: u32 = 5;

pub struct FfmpegDownloader;

#[async_trait]
impl Downloader for FfmpegDownloader {
    async fn download(
        &self,
        episode: EpisodeFile,
        m3u8: &str,
        duration: f64,
        tx: Sender<Progress>,
    ) -> Result<()> {
        let mut last_error = None;

        let id = episode.filename.clone();
        let export_format = "mp4";
        let path = episode.raw_location;
        let tmp_path = path.with_added_extension("tmp");
        let downloaded_path = path.with_added_extension(&export_format);

        if let Some(parent) = tmp_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context(DownloaderError::Crash {
                    downloader: Downloaders::Ffmpeg,
                    msg: format!("Не удалось создать директорию {}", parent.to_string_lossy())
                })?;
        }

        for attempt in 1..=MAX_ATTEMPTS {
            match ffmpeg_download(&tx, &id, m3u8, duration, &export_format, &tmp_path).await {
                Ok(()) => {
                    send_progress(
                        &tx, &id, duration, Status::Finished, duration as u64,
                        Some(format!("Загружено: {}", episode.filename))
                    ).await;
                    tokio::fs::rename(&tmp_path, &downloaded_path).await?;
                    return Ok(());
                }
                Err(e) => {
                    send_progress(
                        &tx, &id, duration, Status::FailAttempt, duration as u64,
                        Some(format!("{} - попытка {attempt}/{MAX_ATTEMPTS}", episode.filename))
                    ).await;
                    last_error = Some(e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
        let err = last_error.unwrap();
        send_progress(&tx, &id, duration, Status::Failed, duration as u64, Some(err.to_string())).await;
        Err(err)
    }
}

async fn ffmpeg_download(
    tx: &Sender<Progress>,
    id: &str,
    m3u8_input: &str,
    m3u8_duration: f64,
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
            "-i", m3u8_input,
            "-c", "copy",
            "-f", export_format,
            "-progress", "pipe:1",
        ])
        .arg(export_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context(DownloaderError::NotFound { downloader: Downloaders::Ffmpeg })?;

    let ffmpeg_stdout = ffmpeg.stdout.take().unwrap();
    let mut lines = BufReader::new(ffmpeg_stdout).lines();

    loop {
        match timeout(Duration::from_secs(EXTERNAL_IDLE_TIMEOUT_SECS), lines.next_line()).await {
            Ok(Ok(Some(line))) => {
                if let Some(time_str) = line.strip_prefix("out_time_ms=") {
                    if let Ok(microseconds) = time_str.parse::<u64>() {
                        send_progress(&tx, id, m3u8_duration, Status::Downloading, (microseconds / 1_000_000) as u64, Some(id.to_string())).await;
                    }
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e.into()),
            Err(_elapsed) => {
                ffmpeg.kill().await.ok();
                anyhow::bail!(DownloaderError::Timeout { downloader: Downloaders::Ffmpeg, timeout: EXTERNAL_IDLE_TIMEOUT_SECS } );
            }
        }
    }

    let status = ffmpeg.wait().await?;

    if !status.success() {
        let mut stderr_output = String::new();
        if let Some(mut stderr) = ffmpeg.stderr.take() {
            stderr.read_to_string(&mut stderr_output).await.ok();
        }
        anyhow::bail!(DownloaderError::Crash { downloader: Downloaders::Ffmpeg, msg: stderr_output });
    }
    Ok(())
}

// TODO: Решить, стоит ли переносить в Progress
async fn send_progress(tx: &Sender<Progress>, id: &str, total_segments: f64, status: Status, downloaded: u64, msg: Option<String>) {
    let _ = tx.send(Progress {
        id: id.to_string(),
        status,
        downloaded_segments: downloaded,
        total_segments,
        msg,
    }).await;
}
