// use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Playlist {
    pub url: String,
    pub total_segments: u32,
    // pub duration: Option<Duration>,
}

pub async fn fetch_playlist_info(client: &reqwest::Client, url: &str) -> Result<Playlist> {
    let playlist = client.get(url).send().await?.text().await?;

    let total_segments: u32 = playlist
        .lines()
        .filter_map(|line| line.strip_prefix("#EXTINF:"))
        .filter_map(|rest| rest.trim_end_matches(',').parse::<u32>().ok())
        .sum();

    Ok(Playlist { url: url.to_string(), total_segments })
}
