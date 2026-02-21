use std::io;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

/// Initializes JSON logging with non-blocking stdout writer.
pub fn init_logger(default_level: &str) -> Result<WorkerGuard, io::Error> {
    let (writer, guard) = tracing_appender::non_blocking(io::stdout());
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level.to_string()));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(writer)
        .json()
        .init();

    Ok(guard)
}
