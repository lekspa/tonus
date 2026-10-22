use std::{fs::File, path::PathBuf};

use color_eyre::Result;
use tracing::Level;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

fn log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("sonus")
}

pub fn init_tracing() -> Result<WorkerGuard> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir)?;

    let file = File::create(dir.join("sonus.log"))?;

    let (non_blocking, guard) = non_blocking(file);

    // By default, the subscriber is configured to log all events with a level of `INFO` or higher,
    // but this can be changed by setting the `RUST_LOG` environment variable.
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(env_filter)
        .with_ansi(false)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    Ok(guard)
}
