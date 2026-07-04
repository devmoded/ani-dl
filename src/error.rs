use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotFound {
    #[error("По запросу \"{0}\" ничего не найдено")]
    Search(String),
    #[error("По запросу \"{0}\" не удалось найти shikimori_id")]
    ShikimoriId(String),
    #[error("Не найдены сезоны")]
    Seasons,
}
