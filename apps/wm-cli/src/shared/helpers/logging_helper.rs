use std::path::PathBuf;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use wm_constants::WM_DIR;

pub fn setup_logging() {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender())
        .with_ansi(false);
    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
}

fn file_appender() -> tracing_appender::rolling::RollingFileAppender {
    tracing_appender::rolling::RollingFileAppender::new(
        tracing_appender::rolling::Rotation::DAILY,
        log_dir(),
        "wm.log",
    )
}

fn log_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(home).join(WM_DIR).join("logs");
    std::fs::create_dir_all(&dir).ok();
    dir
}
