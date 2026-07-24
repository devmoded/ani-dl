// use std::time::Duration;
use anyhow::Result;

pub async fn m3u8_duration(client: &reqwest::Client, m3u8: &str) -> Result<f64> {
    let playlist = client.get(m3u8).send().await?.text().await?;

    let total_segments: f64 = playlist
        .lines()
        .filter_map(|line| line.strip_prefix("#EXTINF:"))
        .filter_map(|rest| rest.trim_end_matches(',').parse::<f64>().ok())
        .sum();

    Ok(total_segments)
}
