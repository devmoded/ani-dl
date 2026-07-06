use anyhow::{Context, Result};
use std::fmt;
use kodik_api::Client as KodikClient;
use reqwest::Client as ReqwestClient;
use kodik_api::search::{SearchQuery, SearchResponse};
use kodik_api::types::{Release, Season, EpisodeUnion};
use serde::Deserialize;

use super::error::NotFound;

pub struct ReleaseItem<'a>(pub &'a Release);

impl fmt::Display for ReleaseItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} ({})",
            self.0.title,
            self.0.translation.title,
            self.0.translation.id
        )
    }
}

#[derive(Deserialize)]
pub struct ShikimoriAnime {
    pub id: u64,
    pub name: String,
    pub russian: Option<String>,
}

impl fmt::Display for ShikimoriAnime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.russian.clone().unwrap_or(self.name.clone()), self.id)
    }
}

#[derive(Clone, Debug)]
pub struct SeasonItem(pub u32, pub Season);

impl fmt::Display for SeasonItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Сезон: {}, Эпизодов:{}", self.0, self.1.episodes.len())
    }
}

#[derive(Clone, Debug)]
pub struct EpisodeItem(pub u32, pub String);

impl fmt::Display for EpisodeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Эпизод {}", self.0)
    }
}

pub async fn search_shikimori(client: &ReqwestClient, query: &str) -> Result<Vec<ShikimoriAnime>> {
    let response: Vec<ShikimoriAnime> = client
        .get("https://shikimori.io/api/animes")
        .query(&[
            ("search", query),
            ("limit", "10"),
            ("order", "popularity"),
        ])
        .send()
        .await?
        .json()
        .await
        .context(NotFound::ShikimoriAnime(query.to_string()))?;
    anyhow::ensure!(!(response.len() == 0), NotFound::ShikimoriAnime(query.to_string()));

    Ok(response)
}

pub async fn search_titles(client: &KodikClient, shikimori_id: &str) -> Result<SearchResponse> {
    let search_response = SearchQuery::new()
        .with_shikimori_id(shikimori_id)
        // .with_title(title)
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

pub async fn get_episodes(season: &Season) -> Result<Vec<EpisodeItem>> {
    let mut episodes: Vec<EpisodeItem> = season
        .episodes
        .iter()
        .filter_map(|(key, ep)| {
            let num: u32 = key.parse().ok()?;
            let link = match ep {
                EpisodeUnion::Link(url) => url.clone(),
                EpisodeUnion::Episode(episode) => episode.link.clone(),
            };
            Some(EpisodeItem(num, link))
        })
        .collect();
    anyhow::ensure!(!(episodes.len() == 0), NotFound::Episodes);
    episodes.sort_by_key(|EpisodeItem(num, _)| *num);
    Ok(episodes)
}
