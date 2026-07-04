use anyhow::{Context, Result};
use kodik_api::Client;
use kodik_api::search::{SearchQuery, SearchResponse};
use kodik_api::types::{Release, Season, EpisodeUnion, TranslationType};

use crate::error::NotFound;

pub async fn search_titles(client: &Client, title: &str) -> Result<SearchResponse> {
    let shikimori_id = get_shikimori_id(&client, title).await?;

    let search_response = SearchQuery::new()
        .with_shikimori_id(&shikimori_id)
        .with_episodes(true)
        .execute(&client)
        .await?;

   Ok(search_response)
}

pub async fn get_shikimori_id(client: &Client, title: &str) -> Result<String> {
    let response = SearchQuery::new()
        .with_title(title)
        .with_limit(1)
        .execute(&client)
        .await?;

    let shikimori_id = response.results.first()
        .ok_or_else(|| NotFound::Search(title.to_string()))?
        .clone()
        .shikimori_id
        .ok_or_else(|| NotFound::ShikimoriId(title.to_string()))?;

    Ok(shikimori_id)
}

pub async fn get_releases(search_response: &SearchResponse) -> Vec<Release> {
    let voice_releases: Vec<Release> = search_response
        .results
        .iter()
        .map(|r| r.clone())
        .collect();
    voice_releases
}

pub async fn get_seasons(releases: &Vec<Release>) -> Result<Vec<(u32, Season)>> {
    let mut seasons: Vec<(u32, Season)> = releases
        .iter()
        .map(|r| r.seasons.as_ref().ok_or_else(|| NotFound::Seasons))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|seasons_map| {
            seasons_map.iter().filter_map(|(k, season)| {
                let num: u32 = k.parse().ok()?;
                Some((num, season.clone()))
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
