use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotFound {
    #[error("По Shikimori ID \"{0}\" ничего не найдено")]
    SearchId(String),
    #[error("Релизы удовлетворяющие запросу не найдены")]
    Releases,
    #[error("По запросу \"{0}\" не удалось найти аниме в Shikimori")]
    ShikimoriAnime(String),
    #[error("Сезоны не найдены")]
    Seasons,
    #[error("Эпизоды не найдены")]
    Episodes,
    #[error("Переменная окружения KODIK_API_KEY не указана")]
    KodikApiKey,
}
