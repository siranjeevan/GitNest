use crate::domain::error::{GitNestError, Result};
use std::path::{Path, PathBuf};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn init_logger(logs_dir: &Path, log_level: &str) -> Result<()> {
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir)?;
    }

    let file_appender = RollingFileAppender::new(Rotation::DAILY, logs_dir, "gitnest.log");

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level.to_lowercase()));

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .try_init()
        .map_err(|e| GitNestError::Io(std::io::Error::other(e)))?;

    Ok(())
}

pub fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
