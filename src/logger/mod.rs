use std::io;
use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

/// Initializes JSON logging.
///
/// - Foreground mode (`daemon = false`): logs to stdout for interactive use.
/// - Daemon mode (`daemon = true`): logs to configured file path.
pub fn init_logger(default_level: &str, log_path: &str, daemon: bool) -> Result<WorkerGuard, io::Error> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level.to_string()));

    if daemon {
        let target_path = Path::new(log_path);
        let parent_dir = target_path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent_dir)?;

        let file_name = target_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("arma.log");

        let appender = tracing_appender::rolling::never(parent_dir, file_name);
        let (writer, guard) = tracing_appender::non_blocking(appender);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(writer)
            .json()
            .init();

        return Ok(guard);
    }

    let (writer, guard) = tracing_appender::non_blocking(io::stdout());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(writer)
        .json()
        .init();

    Ok(guard)
}
