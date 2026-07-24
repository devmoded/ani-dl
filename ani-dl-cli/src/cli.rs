use clap::Parser;
use ani_dl_core::config::{APP_NAME, Downloaders, Engines, Players, Mode, Quality};

/// CLI для просмотра/скачивания аниме
///
/// Для указания ключа Kodik API в файле конфигурации
/// (по умолчанию создаётся в `~/.config/ani-dl/config.toml` при первом запуске)
/// напишите `kodik_api_key = "ваш ключ"`
#[derive(Parser, Debug)]
#[command(name = APP_NAME, version)]
pub struct Cli {
    /// Название аниме-тайтла
    pub query: Option<String>,

    /// Режим работы. По умолчанию: `play`. Доступные варианты: [`play`, `download`]
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// Качество видео. По умолчанию: `hd720p`. Доступные варианты: [`hd720p`, `sd480p`, `low360p`]
    #[arg(short, long)]
    pub quality: Option<Quality>,

    /// Загрузчик. По умолчанию: `ffmpeg`. Доступные варианты: [`ffmpeg`]
    #[arg(short, long)]
    pub downloader: Option<Downloaders>,

    /// Движок. По умолчанию: `kodik`. Доступные варианты: [`kodik`]
    #[arg(short, long)]
    pub engine: Option<Engines>,

    /// Проигрыватель. По умолчанию: `mpv`. Доступные варианты: [`mpv`]
    #[arg(short, long)]
    pub player: Option<Players>,

    /// Путь сохранения аниме (только при скачивании)
    #[arg(short, long, default_value = ".")]
    pub output_dir: String,

    /// Сгенерировать Python скрипт для получения ключа KODIK API
    #[arg(long)]
    pub gen_kodik_key_script: bool,
}
