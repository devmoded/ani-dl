mod kodik;

use async_trait::async_trait;
use anyhow::Result;
use crate::types::Response;
use crate::config::{Engines, Quality};

#[async_trait]
pub trait Engine {
    async fn search(&self, shikimori_id: &str) -> Result<Response>;
    async fn resolve_link(&self, url: &str, quality: &Quality) -> Result<String>;
}

pub async fn search(
    engine: &Engines,
    shikimori_id: &str,
    key: impl Into<String>
) -> Result<Response> {
    let engine = match engine {
        Engines::Kodik => kodik::Kodik::new(key),
    };

    let response = engine.search(shikimori_id).await?;
    Ok(response)
}

pub async fn resolve_link(
    engine: &Engines,
    url: &str,
    quality: &Quality,
    key: impl Into<String>
) -> Result<String> {
    let engine = match engine {
        Engines::Kodik => kodik::Kodik::new(key),
    };

    let link = engine.resolve_link(url, quality).await?;
    Ok(link)
}
