use serde::Deserialize;

/// Runtime configuration loaded from `config.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub filter_pack: FilterPackConfig,
}

/// Server binding configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    pub port: u16,
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub path: String,
}

/// Directory-based filter-pack configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct FilterPackConfig {
    pub dir: String,
    #[serde(default)]
    pub profile: Option<String>,
}
