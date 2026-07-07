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
    #[error("Ключ Kodik API не указан")]
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
pub enum ShikimoriError {
    #[error("По запросу \"{0}\" не удалось найти аниме в Shikimori")]
    NotFound(String),
    #[error("URL для API не задан")]
    ApiUrlNotSet,
}

#[derive(Debug, Error)]
pub enum Crash {
    #[error("Работа mpv завершилась с ошибкой")]
    Mpv,
    #[error("Работа ffmpeg завершилась с ошибкой: {0}")]
    Ffmpeg(String),
    #[error("Не удалось распарсить ответ Shikimori")]
    ShikimoriParse,
    #[error("Закончилось время ({0} секунд) на выполнение команды")]
    Timeout(u64),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Не удалось распарсить конфиг: \"{0}\"")]
    Parse(String),
    #[error("Не удалось прочитать файл конфига: \"{0}\"")]
    Read(String),
    #[error("Не удалось записать файл конфига: \"{0}\"")]
    Write(String),
    #[error("Не удалось создать директорию для конфига: \"{0}\"")]
    CreateDir(String),
    #[error("Не удалось найти конфиг")]
    NotFound,
}
