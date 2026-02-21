use serde::{Deserialize, Serialize};

/// Deserialized filter pack payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilterPack {
    pub version: String,
    pub last_updated: String,
    #[serde(default)]
    pub deny_keywords: Vec<String>,
    #[serde(default)]
    pub deny_patterns: Vec<String>,
    #[serde(default)]
    pub allow_keywords: Vec<String>,
    pub settings: FilterPackSettings,
}

/// Configurable threshold settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilterPackSettings {
    pub sensitivity_score: u32,
}

impl FilterPack {
    /// Returns the configured sensitivity score.
    pub fn sensitivity_score(&self) -> u32 {
        self.settings.sensitivity_score
    }
}
