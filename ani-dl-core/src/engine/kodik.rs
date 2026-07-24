use anyhow::{Context, Result};
use async_trait::async_trait;
use kodik_api::{Client as ApiClient, search::{SearchResponse, SearchQuery}, types::EpisodeUnion};
use kodik_parser::reqwest::Client as ResolveClient;
use crate::engine::{Engine, Engines, Quality};
use crate::types::{Response, Release, Translation, Season, Episode};
use crate::error::EngineError;

#[derive(Debug)]
pub struct Kodik {
    pub api_client: ApiClient,
    pub resolve_client: ResolveClient,
}

impl Kodik {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            api_client: ApiClient::new(key),
            resolve_client: ResolveClient::new(),
        }
    }
}

#[async_trait]
impl Engine for Kodik {
    async fn search(&self, shikimori_id: &str) -> Result<Response> {
        let response = SearchQuery::new()
            .with_shikimori_id(shikimori_id)
            .with_episodes(true)
            .execute(&self.api_client)
            .await
            .context(EngineError::SearchError { engine: Engines::Kodik })?;
        Ok(Response::from(&response))
    }

    async fn resolve_link(&self, link: &str, quality: &Quality) -> Result<String> {
        let url = format!("https:{link}");
        let links = kodik_parser::parse(&self.resolve_client, &url).await?.links;

        let m3u8 = match quality {
            Quality::Hd720p => links.quality_720
                .first()
                .context(EngineError::ResolveError { url, engine: Engines::Kodik })?
                .src
                .clone(),
            Quality::Sd480p => links.quality_480
                .first()
                .context(EngineError::ResolveError { url, engine: Engines::Kodik })?
                .src
                .clone(),
            Quality::Low360p => links.quality_360
                .first()
                .context(EngineError::ResolveError { url, engine: Engines::Kodik })?
                .src
                .clone(),
        };

        Ok(m3u8)
    }
}

impl From<&SearchResponse> for Response {
    fn from(r: &SearchResponse) -> Self {
        Response {
            releases: r.results.iter().map(|r| Release::from(r)).collect(),
        }
    }
}

impl From<&kodik_api::types::Release> for Release {
    fn from(r: &kodik_api::types::Release) -> Self {
        Release {
            title: r.title.clone(),
            shikimori_id: r.shikimori_id.clone(),
            translation: Translation::from(&r.translation),
            seasons: match &r.seasons {
                Some(seasons) => {
                    Some(seasons.iter().map(|(_, s)| Season::from(s)).collect())
                }
                None => None,
            },
        }
    }
}

impl From<&kodik_api::types::Translation> for Translation {
    fn from(t: &kodik_api::types::Translation) -> Self {
        Translation {
            title: t.title.clone(),
            id: t.id as u32,
        }
    }
}

impl From<&kodik_api::types::Season> for Season {
    fn from(s: &kodik_api::types::Season) -> Self {
        let mut episodes: Vec<Episode> = s.episodes
            .iter()
            .filter_map(|(key, ep)| {
                let num: u32 = key.parse().ok()?;
                let link = match ep {
                    EpisodeUnion::Link(url) => url.clone(),
                    EpisodeUnion::Episode(episode) => episode.link.clone(),
                };
                Some(Episode { num, link })
            })
            .collect();

        episodes.sort_by_key(|ep| ep.num);

        Season {
            title: s.title.clone(),
            episodes,
        }
    }
}
