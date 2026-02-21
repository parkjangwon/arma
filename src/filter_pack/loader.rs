use std::cmp::Ordering;
use std::fmt;
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
pub async fn load_merged_filter_pack_dir(
    dir_path: &Path,
    profile: Option<&str>,
) -> Result<FilterPack, LoaderError> {
    let mut files = collect_rule_files(dir_path)?;
    files.sort_by(compare_file_name_then_path);
    files.retain(|path| should_include_rule_file(path, profile));

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

fn should_include_rule_file(path: &Path, profile: Option<&str>) -> bool {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if !file_name.contains("-profile-") {
        return true;
    }

    match profile {
        Some(value) => file_name.contains(&format!("-profile-{value}")),
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::load_merged_filter_pack_dir;

    fn temp_test_dir() -> std::path::PathBuf {
        for attempt in 0..32 {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("valid unix time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "arma-loader-test-{}-{attempt}-{}",
                std::process::id(),
                unique
            ));
            if std::fs::create_dir(&path).is_ok() {
                return path;
            }
        }

        panic!("failed to create temp test directory");
    }

    #[tokio::test]
    async fn excludes_profile_files_when_profile_is_not_set() {
        let dir = temp_test_dir();
        std::fs::write(
            dir.join("00-core.yaml"),
            "version: \"1\"\nlast_updated: \"today\"\nallow_keywords: [\"core\"]\nsettings:\n  sensitivity_score: 70\n",
        )
        .expect("write core rule");
        std::fs::write(
            dir.join("10-profile-strict.yaml"),
            "version: \"1\"\nlast_updated: \"today\"\nallow_keywords: [\"strict\"]\nsettings:\n  sensitivity_score: 90\n",
        )
        .expect("write strict profile rule");

        let merged = load_merged_filter_pack_dir(&dir, None)
            .await
            .expect("load merged pack without profile");
        assert!(merged.allow_keywords.iter().any(|value| value == "core"));
        assert!(!merged.allow_keywords.iter().any(|value| value == "strict"));

        std::fs::remove_dir_all(dir).expect("cleanup temp test dir");
    }

    #[tokio::test]
    async fn includes_only_selected_profile_file() {
        let dir = temp_test_dir();
        std::fs::write(
            dir.join("00-core.yaml"),
            "version: \"1\"\nlast_updated: \"today\"\nallow_keywords: [\"core\"]\nsettings:\n  sensitivity_score: 70\n",
        )
        .expect("write core rule");
        std::fs::write(
            dir.join("10-profile-balanced.yaml"),
            "version: \"1\"\nlast_updated: \"today\"\nallow_keywords: [\"balanced\"]\nsettings:\n  sensitivity_score: 75\n",
        )
        .expect("write balanced profile rule");
        std::fs::write(
            dir.join("10-profile-strict.yaml"),
            "version: \"1\"\nlast_updated: \"today\"\nallow_keywords: [\"strict\"]\nsettings:\n  sensitivity_score: 90\n",
        )
        .expect("write strict profile rule");

        let merged = load_merged_filter_pack_dir(&dir, Some("strict"))
            .await
            .expect("load merged pack with strict profile");
        assert!(merged.allow_keywords.iter().any(|value| value == "core"));
        assert!(merged.allow_keywords.iter().any(|value| value == "strict"));
        assert!(!merged.allow_keywords.iter().any(|value| value == "balanced"));

        std::fs::remove_dir_all(dir).expect("cleanup temp test dir");
    }
}
