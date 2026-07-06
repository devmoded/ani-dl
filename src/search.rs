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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.russian.clone().unwrap_or(self.name.clone()), self.id)
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
        .map(|r| r)
        .collect();
    anyhow::ensure!(!(releases.len() == 0), NotFound::Releases);
    Ok(releases)
}

pub async fn get_seasons(releases: &Vec<Release>) -> Result<Vec<(u32, &Season)>> {
    let mut seasons: Vec<(u32, &Season)> = releases
        .iter()
        .map(|r| r.seasons.as_ref().ok_or_else(|| NotFound::Seasons))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|seasons_map| {
            seasons_map.iter().filter_map(|(k, season)| {
                let num: u32 = k.parse().ok()?;
                Some((num, season))
            })
        })
        .collect();

    seasons.sort_by_key(|(num, _)| *num);
    Ok(seasons)
}

pub async fn get_episodes(season: &Season) -> Vec<(u32, &str)> {
    let mut items: Vec<(u32, &str)> = season
        .episodes
        .iter()
        .filter_map(|(key, ep)| {
            let num: u32 = key.parse().ok()?;
            let link = match ep {
                EpisodeUnion::Link(url) => url.as_str(),
                EpisodeUnion::Episode(episode) => episode.link.as_str(),
            };
            Some((num, link))
        })
        .collect();
    items.sort_by_key(|(num, _)| *num);
    items
}
