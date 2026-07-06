use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotFound {
    #[error("По Shikimori ID \"{0}\" ничего не найдено")]
    SearchId(String),
    #[error("Релизы удовлетворяющие запросу не найдены")]
    Releases,
    #[error("Сезоны не найдены")]
    Seasons,
    #[error("Эпизоды не найдены")]
    Episodes,
    #[error("Переменная окружения KODIK_API_KEY не указана")]
    KodikApiKey,
    #[error("Не удалось найти URL для разрешения 720p")]
    Url720p,
    #[error("Не удалось найти mpv - он установлен?")]
    Mpv,
    #[error("Не удалось найти ffmpeg - он установлен?")]
    Ffmpeg,
    #[error("Ничего не выбрано")]
    Select,
}

#[derive(Debug, Error)]
pub enum Shikimori {
    #[error("По запросу \"{0}\" не удалось найти аниме в Shikimori")]
    NotFound(String),
    #[error("URL для API не задан")]
    ApiUrlNotSet,
}

#[derive(Debug, Error)]
pub enum Crash {
    #[error("Работа mpv завершилась с ошибкой")]
    Mpv,
    #[error("Работа ffmpeg завершилась с ошибкой")]
    Ffmpeg,
    #[error("Не удалось распарсить ответ Shikimori")]
    ShikimoriParse,
}
