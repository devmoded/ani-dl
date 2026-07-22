use thiserror::Error;
use crate::engines::Engines;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Не удалось получить ссылку на m3u8 плейлист из url: \"{url}\"")]
    ResolveError { url: String },
    #[error("Не удалось ничего найти в базе {engine:?}")]
    SearchError { engine: Engines },
}

#[derive(Error, Debug)]
pub enum ShikimoriError {
    #[error("API url не указан")]
    ApiNotSet,
    #[error("По запросу \"{query}\" в базе Shikimori ничего не найдено")]
    NotFound { query: String },
    #[error("Не удалось распарсить ответ Shikimori")]
    ParseCrash,
}
