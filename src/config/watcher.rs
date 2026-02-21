use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::config::{load_app_config, load_filter_pack, resolve_filter_pack_dir, SharedEngine};
use crate::core::engine::FilterEngine;
use crate::filter_pack::FilterPack;

/// Hot-reload watcher setup or runtime error.
#[derive(Debug)]
pub enum WatcherError {
    Notify(notify::Error),
    EventChannelClosed,
}

impl fmt::Display for WatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Notify(err) => write!(f, "notify error: {err}"),
            Self::EventChannelClosed => write!(f, "notify event channel closed"),
        }
    }
}

impl std::error::Error for WatcherError {}

impl From<notify::Error> for WatcherError {
    fn from(value: notify::Error) -> Self {
        Self::Notify(value)
    }
}

/// Runs file watch worker and atomically swaps engine state on valid updates.
pub async fn run_hot_reload_worker(
    shared_engine: SharedEngine,
    config_path: PathBuf,
    initial_filter_pack_dir: PathBuf,
    initial_filter_pack_digest: u64,
) -> Result<(), WatcherError> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Result<Event, notify::Error>>();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            if tx.send(event).is_err() {
                tracing::warn!("hot-reload channel dropped");
            }
        },
        notify::Config::default(),
    )?;

    watch_path_non_recursive(&mut watcher, &config_path)?;
    watch_path_recursive(&mut watcher, &initial_filter_pack_dir)?;

    let normalized_config_path = absolute_path(&config_path);
    let mut active_filter_pack_dir = absolute_path(&initial_filter_pack_dir);
    let mut active_filter_pack_digest = initial_filter_pack_digest;

    loop {
        let first_event = match rx.recv().await {
            Some(event) => event,
            None => return Err(WatcherError::EventChannelClosed),
        };

        let mut should_reload = event_targets_path(
            first_event,
            &normalized_config_path,
            &active_filter_pack_dir,
        );

        tokio::time::sleep(Duration::from_millis(150)).await;
        while let Ok(next_event) = rx.try_recv() {
            should_reload |= event_targets_path(
                next_event,
                &normalized_config_path,
                &active_filter_pack_dir,
            );
        }

        if !should_reload {
            continue;
        }

        let next_filter_pack_dir = trigger_hot_reload(
            &shared_engine,
            &normalized_config_path,
            &active_filter_pack_dir,
            &mut active_filter_pack_digest,
        )
        .await;

        if let Some(path) = next_filter_pack_dir {
            if let Err(error) = watch_path_recursive(&mut watcher, &path) {
                tracing::warn!(error = %error, "failed to watch new filter-pack directory");
            }
            active_filter_pack_dir = path;
        }
    }
}

/// Triggers one safe hot-reload cycle and returns updated filter-pack path on success.
pub async fn trigger_hot_reload(
    shared_engine: &SharedEngine,
    config_path: &Path,
    current_filter_pack_dir: &Path,
    current_filter_pack_digest: &mut u64,
) -> Option<PathBuf> {
    let app_config = match load_app_config(config_path).await {
        Ok(config) => config,
        Err(error) => {
            tracing::warn!(error = %error, "config hot-reload parse failed; keeping current engine");
            return None;
        }
    };

    let next_filter_pack_dir = absolute_path(&resolve_filter_pack_dir(&app_config, config_path));
    let filter_pack_dir = if next_filter_pack_dir.exists() {
        next_filter_pack_dir
    } else {
        absolute_path(current_filter_pack_dir)
    };

    let filter_pack = match load_filter_pack(&filter_pack_dir, app_config.filter_pack.profile.as_deref()).await {
        Ok(pack) => pack,
        Err(error) => {
            tracing::warn!(error = %error, "filter-pack parse failed; keeping current engine");
            return None;
        }
    };

    let next_digest = filter_pack_digest(&filter_pack);
    if next_digest == *current_filter_pack_digest {
        tracing::debug!(
            filter_pack_dir = %filter_pack_dir.display(),
            "hot-reload skipped because merged rules are unchanged"
        );
        return Some(filter_pack_dir);
    }

    let new_engine = match FilterEngine::new(&filter_pack) {
        Ok(engine) => engine,
        Err(error) => {
            tracing::warn!(error = %error, "filter engine build failed; keeping current engine");
            return None;
        }
    };

    let next_version = new_engine.filter_pack_version().to_string();
    {
        let mut guard = shared_engine.write().await;
        *guard = new_engine;
    }

    *current_filter_pack_digest = next_digest;

    tracing::info!(filter_pack_version = %next_version, "filter engine hot-reload applied");
    Some(filter_pack_dir)
}

fn watch_path_non_recursive(
    watcher: &mut RecommendedWatcher,
    path: &Path,
) -> Result<(), notify::Error> {
    let watch_path = path
        .parent()
        .filter(|candidate| !candidate.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    watcher.watch(watch_path, RecursiveMode::NonRecursive)
}

fn watch_path_recursive(watcher: &mut RecommendedWatcher, path: &Path) -> Result<(), notify::Error> {
    watcher.watch(path, RecursiveMode::Recursive)
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    match std::env::current_dir() {
        Ok(current_dir) => current_dir.join(path),
        Err(_) => path.to_path_buf(),
    }
}

fn event_targets_path(
    event: Result<Event, notify::Error>,
    config_path: &Path,
    filter_pack_dir: &Path,
) -> bool {
    match event {
        Ok(payload) => {
            tracing::debug!(event_kind = ?payload.kind, paths = ?payload.paths, "watcher event received");
            payload.paths.iter().map(|path| absolute_path(path)).any(|path| {
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
                .unwrap_or(false);

            path == config_path
                || (path.starts_with(filter_pack_dir) && extension)
                || path.file_name() == config_path.file_name()
                || path == filter_pack_dir
            })
        }
        Err(error) => {
            tracing::warn!(error = %error, "watcher event error");
            false
        }
    }
}

/// Computes deterministic digest for merged filter-pack content.
pub fn filter_pack_digest(filter_pack: &FilterPack) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    filter_pack.version.hash(&mut hasher);
    filter_pack.last_updated.hash(&mut hasher);
    filter_pack.deny_keywords.hash(&mut hasher);
    filter_pack.deny_patterns.hash(&mut hasher);
    filter_pack.allow_keywords.hash(&mut hasher);
    filter_pack.sensitivity_score().hash(&mut hasher);
    hasher.finish()
}
