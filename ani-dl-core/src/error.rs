use thiserror::Error;
use crate::config::Engines;
use crate::types::Release;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Не удалось получить ссылку на m3u8 плейлист из url: \"{url}\" для {engine}")]
    ResolveError { url: String, engine: Engines },
    #[error("Не удалось ничего найти в базе {engine}")]
    SearchError { engine: Engines },
    #[error("В базе {engine} не найдены сезоны для {release}")]
    NotFoundSeasons { engine: Engines, release: Release },
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Эпизоды не выбраны")]
    EpisodesNotSelected,
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

// TODO: Сделать универсальный DownloaderError
#[derive(Error, Debug)]
pub enum FfmpegError {
    #[error("Время на выполнение: {0} (секунд) закончилось")]
    Timeout(u64),
    #[error("Не удалось найти исполняемый файл ffmpeg")]
    NotFound,
    #[error("Работа ffmpeg завершилась с ошибкой: \"{msg}\"")]
    Crash { msg: String },
}

// TODO: Сделать универсальный PlayerError
#[derive(Error, Debug)]
pub enum MpvError {
    #[error("Не удалось найти исполняемый файл mpv")]
    NotFound,
    #[error("Работа mpv завершилась с ошибкой")]
    Crash,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("В файле конфигурации не указан URL для Shikimori API")]
    ShikimoriApiUrlNotSet,
    #[error("В файле конфигурации не указан ключ для Kodik")]
    KodikApiKeyNotSet,
    #[error("Не удалось найти каталог конфигураций")]
    NotFound,
    #[error("Не удалось записать файл: {path}")]
    Write { path: String },
    #[error("Не удалось прочитать файл: {path}")]
    Read { path: String },
    #[error("Не удалось распарсить файл: {path}")]
    Parse { path: String },
    #[error("Не удалось создать директорию: {path}")]
    CreateDir { path: String },
}
