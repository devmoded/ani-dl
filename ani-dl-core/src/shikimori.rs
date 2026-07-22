use std::fmt;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use crate::config::{APP_NAME, APP_VERSION};
use crate::error::ShikimoriError;

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
    api: String,
    client: Client,
}

impl Search {
    pub fn new(api_url: &str) -> Result<Self> {
        Ok(Self {
            api: api_url.to_string(),
            client: Client::builder()
                .user_agent(format!("{APP_NAME}-rust/{APP_VERSION}"))
                .build()?,
        })
    }

    pub async fn execute(&self, query: &str, limit: u32) -> Result<Vec<Anime>> {
        let response: Vec<Anime> = self.client
            .get(&self.api)
            .query(&[
                ("search", query),
                ("limit", &limit.to_string()),
                ("order", "popularity"),
            ])
            .send()
            .await
            .context(ShikimoriError::NotFound { query: query.to_string() })?
            .json()
            .await
            .context(ShikimoriError::ParseCrash)?;
        anyhow::ensure!(!(response.len() == 0), ShikimoriError::NotFound { query: query.to_string() });

        Ok(response)
    }
}
