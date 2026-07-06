use std::fmt;
use anyhow::{Context, Result};
use reqwest::Client as ReqwestClient;
use serde::Deserialize;

use crate::error::{Shikimori, Crash};

#[derive(Deserialize, Debug)]
pub struct Anime {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub russian: String,
}

impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = if self.russian.is_empty() {&self.name} else {&self.russian};
        write!(f, "{} ({})", title, self.id)
    }
}

pub struct Search {
    api_url: Option<String>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            api_url: None,
        }
    }

    pub fn with_api_url(&mut self, url: &str) -> &mut Self {
        self.api_url = Some(url.to_string());
        self
    }


    pub async fn execute(&self, reqwest_client: &ReqwestClient, query: &str, limit: u32) -> Result<Vec<Anime>> {
        let response: Vec<Anime> = reqwest_client
            .get(self.api_url.clone().context(Shikimori::ApiUrlNotSet)?)
            .query(&[
                ("search", query),
                ("limit", &limit.to_string()),
                ("order", "popularity"),
            ])
            .send()
            .await
            .context(Shikimori::NotFound(query.to_string()))?
            .json()
            .await
            .context(Crash::ShikimoriParse)?;
        anyhow::ensure!(!(response.len() == 0), Shikimori::NotFound(query.to_string()));

        Ok(response)
    }
}
