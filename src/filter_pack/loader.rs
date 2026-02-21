use std::fmt;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::filter_pack::models::{FilterPack, FilterPackSettings};

/// Directory rule loader errors.
#[derive(Debug)]
pub enum LoaderError {
    Io(std::io::Error),
    Yaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    NoRuleFiles(PathBuf),
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Yaml { path, source } => {
                write!(f, "YAML parse error in {}: {source}", path.display())
            }
            Self::NoRuleFiles(path) => write!(
                f,
                "no YAML rule files found in directory: {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Deserialize)]
struct PartialFilterPack {
    version: Option<String>,
    last_updated: Option<String>,
    #[serde(default)]
    deny_keywords: Vec<String>,
    #[serde(default)]
    deny_patterns: Vec<String>,
    #[serde(default)]
    allow_keywords: Vec<String>,
    settings: Option<PartialSettings>,
}

#[derive(Debug, Deserialize)]
struct PartialSettings {
    sensitivity_score: Option<u32>,
}

/// Loads and merges all rule YAML files from a directory.
pub async fn load_merged_filter_pack_dir(dir_path: &Path) -> Result<FilterPack, LoaderError> {
    let mut files = collect_rule_files(dir_path)?;
    files.sort_by(compare_file_name_then_path);

    tracing::debug!(
        rule_dir = %dir_path.display(),
        file_count = files.len(),
        "scanned filter-pack directory"
    );

    if files.is_empty() {
        return Err(LoaderError::NoRuleFiles(dir_path.to_path_buf()));
    }

    let mut merged = FilterPack {
        version: "0.0.0".to_string(),
        last_updated: "unknown".to_string(),
        deny_keywords: Vec::new(),
        deny_patterns: Vec::new(),
        allow_keywords: Vec::new(),
        settings: FilterPackSettings {
            sensitivity_score: 70,
        },
    };

    for file_path in files {
        tracing::debug!(file = %file_path.display(), "merging filter-pack file");
        let raw = tokio::fs::read_to_string(&file_path).await?;
        let partial = serde_yaml::from_str::<PartialFilterPack>(&raw).map_err(|source| {
            LoaderError::Yaml {
                path: file_path.clone(),
                source,
            }
        })?;

        merged.deny_keywords.extend(partial.deny_keywords);
        merged.deny_patterns.extend(partial.deny_patterns);
        merged.allow_keywords.extend(partial.allow_keywords);

        if let Some(version) = partial.version {
            merged.version = version;
        }
        if let Some(last_updated) = partial.last_updated {
            merged.last_updated = last_updated;
        }
        if let Some(settings) = partial.settings {
            if let Some(score) = settings.sensitivity_score {
                merged.settings.sensitivity_score = score;
            }
        }
    }

    Ok(merged)
}

fn collect_rule_files(dir_path: &Path) -> Result<Vec<PathBuf>, LoaderError> {
    let mut paths = Vec::new();
    let mut stack = vec![dir_path.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if is_yaml_file(&path) {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}

fn is_yaml_file(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(value) => {
            let lowered = value.to_ascii_lowercase();
            lowered == "yaml" || lowered == "yml"
        }
        None => false,
    }
}

fn compare_file_name_then_path(left: &PathBuf, right: &PathBuf) -> Ordering {
    let left_name = left
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let right_name = right
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    match left_name.cmp(right_name) {
        Ordering::Equal => left.cmp(right),
        value => value,
    }
}
