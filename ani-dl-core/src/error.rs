use thiserror::Error;
use crate::engine::Engines;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Не удалось получить ссылку на m3u8 плейлист из url: \"{url}\" для {engine:?}")]
    ResolveError { url: String, engine: Engines },
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

#[derive(Error, Debug)]
pub enum FfmpegError {
    #[error("Время на выполнение: {0} (секунд) закончилось")]
    Timeout(u64),
    #[error("Не удалось найти исполняемый файл ffmpeg")]
    NotFound,
    #[error("Работа ffmpeg завершилась с ошибкой: \"{msg}\"")]
    Crash { msg: String },
}
