use anyhow::{Context, Result};
use std::path::PathBuf;
use kodik_api::Client as KodikClient;
use kodik_api::search::{SearchQuery, SearchResponse};
use kodik_api::types::{Release, Season, EpisodeUnion};

use crate::error::NotFound;
use crate::types::{EpisodeItem, SeasonItem, ReleaseItem};

pub async fn search_titles(client: &KodikClient, shikimori_id: &str) -> Result<SearchResponse> {
    let search_response = SearchQuery::new()
        .with_shikimori_id(shikimori_id)
        .with_episodes(true)
        .execute(&client)
        .await
        .context(NotFound::SearchId(shikimori_id.to_string()))?;

   Ok(search_response)
}

pub async fn get_releases(search_response: &SearchResponse) -> Result<Vec<&Release>> {
    let releases: Vec<&Release> = search_response
        .results
        .iter()
        .collect();
    anyhow::ensure!(!(releases.len() == 0), NotFound::Releases);
    Ok(releases)
}

pub async fn get_seasons(release: Release) -> Result<Vec<SeasonItem>> {
    let mut seasons: Vec<SeasonItem> = release
        .seasons.context(NotFound::Seasons)?
        .iter()
        .filter_map(|(k, season)| {
            let num: u32 = k.parse().ok()?;
            Some(SeasonItem(num, season.clone()))
        })
        .collect();
    anyhow::ensure!(!(seasons.len() == 0), NotFound::Seasons);
    seasons.sort_by_key(|SeasonItem(num, _)| *num);
    Ok(seasons)
}

pub async fn get_episodes(season: &Season, release: &ReleaseItem<'_>, output: &PathBuf) -> Result<Vec<EpisodeItem>> {
    let mut episodes: Vec<EpisodeItem> = season
        .episodes
        .iter()
        .filter_map(|(key, ep)| {
            let num: u32 = key.parse().ok()?;
            let link = match ep {
                EpisodeUnion::Link(url) => url.clone(),
                EpisodeUnion::Episode(episode) => episode.link.clone(),
            };
            let filename = format!("EP{:02} - {}.mp4", num, release);
            Some(EpisodeItem{
                num: num,
                url: link,
                filename: filename,
                output_dir: output.clone()
            })
        })
        .collect();
    anyhow::ensure!(!(episodes.len() == 0), NotFound::Episodes);
    episodes.sort_by_key(|EpisodeItem{num, url: _, filename: _, output_dir: _}| *num);
    Ok(episodes)
}
