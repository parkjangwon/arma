use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::engine::FilterEngine;
use crate::filter_pack::loader::{LoaderError, load_merged_filter_pack_dir};
use crate::filter_pack::FilterPack;

pub mod models;

pub mod watcher;

pub use models::AppConfig;

/// Shared filter engine handle for high read-concurrency access.
pub type SharedEngine = Arc<RwLock<FilterEngine>>;

/// Configuration or loading error.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Loader(LoaderError),
    Engine(crate::core::engine::EngineError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Yaml(err) => write!(f, "YAML parse error: {err}"),
            Self::Loader(err) => write!(f, "filter-pack directory load error: {err}"),
            Self::Engine(err) => write!(f, "engine initialization error: {err}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Yaml(value)
    }
}

impl From<crate::core::engine::EngineError> for ConfigError {
    fn from(value: crate::core::engine::EngineError) -> Self {
        Self::Engine(value)
    }
}

impl From<LoaderError> for ConfigError {
    fn from(value: LoaderError) -> Self {
        Self::Loader(value)
    }
}

/// Loads app runtime configuration from YAML file.
pub async fn load_app_config(path: &Path) -> Result<AppConfig, ConfigError> {
    let raw = tokio::fs::read_to_string(path).await?;
    let config = serde_yaml::from_str::<AppConfig>(&raw)?;
    Ok(config)
}

/// Loads merged filter-pack data from directory.
pub async fn load_filter_pack(dir_path: &Path) -> Result<FilterPack, ConfigError> {
    let pack = load_merged_filter_pack_dir(dir_path).await?;
    Ok(pack)
}

/// Resolves the filter-pack directory from config, relative to config location when needed.
pub fn resolve_filter_pack_dir(config: &AppConfig, config_path: &Path) -> PathBuf {
    let configured = PathBuf::from(&config.filter_pack.dir);
    if configured.is_absolute() {
        return configured;
    }

    match config_path.parent() {
        Some(parent) => parent.join(configured),
        None => PathBuf::from(".").join(configured),
    }
}
