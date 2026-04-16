//! Logging setup via `tracing-subscriber`.
//!
//! Writes to both stderr (during development) and a rolling file under
//! `[logging].log_dir`. The file name is `silentwitness.log`, rotated daily.

use std::path::Path;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::LoggingConfig;

pub fn init(cfg: &LoggingConfig, tag: &str) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    if !cfg.enabled {
        return Ok(None);
    }
    let filter = EnvFilter::try_new(&cfg.level).unwrap_or_else(|_| EnvFilter::new("info"));

    let stderr_layer = fmt::layer().with_target(false).with_ansi(false).with_writer(std::io::stderr);

    if cfg.to_file {
        let dir = Path::new(&cfg.log_dir);
        std::fs::create_dir_all(dir).ok();
        let file_appender = tracing_appender::rolling::daily(dir, format!("silentwitness-{tag}.log"));
        let (nb, guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = fmt::layer().with_target(false).with_ansi(false).with_writer(nb);
        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .with(file_layer)
            .try_init()
            .ok();
        Ok(Some(guard))
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .try_init()
            .ok();
        Ok(None)
    }
}
