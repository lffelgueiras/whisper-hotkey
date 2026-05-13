use crate::models::app_data_dir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init() -> tracing_appender::non_blocking::WorkerGuard {
    let dir = app_data_dir().join("logs");
    std::fs::create_dir_all(&dir).ok();
    let appender = RollingFileAppender::new(Rotation::DAILY, dir, "app.log");
    let (nb, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,whisper_hotkey=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_writer(nb))
        .with(fmt::layer().with_target(false))
        .init();
    tracing::info!("logging initialized");
    guard
}

pub fn logs_dir() -> std::path::PathBuf {
    app_data_dir().join("logs")
}
