use thiserror::Error;
use crate::config::{Engines, Players, Downloaders};
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

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("Время на выполнение {downloader}: {timeout} (секунд) закончилось")]
    Timeout { downloader: Downloaders, timeout: u64 },
    #[error("Не удалось найти исполняемый файл {downloader}")]
    NotFound { downloader: Downloaders },
    #[error("Работа {downloader} завершилась с ошибкой: \"{msg}\"")]
    Crash { downloader: Downloaders, msg: String },
}

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("Не удалось найти исполняемый файл плеера {player}")]
    NotFound { player: Players },
    #[error("Работа плеера {player} завершилась с ошибкой")]
    Crash { player: Players },
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
