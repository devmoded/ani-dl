use async_trait::async_trait;
use anyhow::Result;
use crate::types::Response;

pub mod kodik;

#[derive(Debug)]
pub enum Engines {
    Kodik,
}

#[async_trait]
pub trait Engine {
    async fn search(&self, shikimori_id: &str) -> Result<Response>;
    async fn resolve_link(&self, url: &str, quality: &Quality) -> Result<String>;
}

pub enum Quality {
    Hd720p,
    Sd480p,
    Low360p,
}
