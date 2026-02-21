use std::fmt;

use aho_corasick::AhoCorasick;
use regex::Regex;

use crate::core::normalizer::normalize_for_detection;
use crate::filter_pack::FilterPack;

/// Prompt validation output from the core engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_safe: bool,
    pub reason: String,
    pub score: u32,
}

/// Engine initialization or runtime validation errors.
#[derive(Debug)]
pub enum EngineError {
    AhoCorasickBuild(aho_corasick::BuildError),
    RegexBuild(regex::Error),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AhoCorasickBuild(err) => {
                write!(f, "failed to build aho-corasick automaton: {err}")
            }
            Self::RegexBuild(err) => write!(f, "failed to compile regex pattern: {err}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<aho_corasick::BuildError> for EngineError {
    fn from(value: aho_corasick::BuildError) -> Self {
        Self::AhoCorasickBuild(value)
    }
}

impl From<regex::Error> for EngineError {
    fn from(value: regex::Error) -> Self {
        Self::RegexBuild(value)
    }
}

/// Precompiled high-performance filter engine.
pub struct FilterEngine {
    deny_keyword_automaton: AhoCorasick,
    deny_keywords: Vec<String>,
    deny_patterns: Vec<Regex>,
    allow_keywords: Vec<String>,
    filter_pack_version: String,
    sensitivity_score: u32,
}

impl FilterEngine {
    /// Builds a filter engine with one-time compiled matchers.
    pub fn new(filter_pack: &FilterPack) -> Result<Self, EngineError> {
        let deny_keywords: Vec<String> = filter_pack
            .deny_keywords
            .iter()
            .map(|value| normalize_for_detection(value))
            .filter(|value| !value.is_empty())
            .collect();

        let allow_keywords: Vec<String> = filter_pack
            .allow_keywords
            .iter()
            .map(|value| normalize_for_detection(value))
            .filter(|value| !value.is_empty())
            .collect();

        let deny_patterns: Vec<Regex> = filter_pack
            .deny_patterns
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>()?;

        let deny_keyword_automaton = AhoCorasick::new(&deny_keywords)?;

        Ok(Self {
            deny_keyword_automaton,
            deny_keywords,
            deny_patterns,
            allow_keywords,
            filter_pack_version: filter_pack.version.clone(),
            sensitivity_score: filter_pack.sensitivity_score(),
        })
    }

    /// Returns currently loaded filter pack version.
    pub fn filter_pack_version(&self) -> &str {
        &self.filter_pack_version
    }

    /// Validates a prompt through normalization, allow-list, deny-keyword, and regex checks.
    pub fn validate(&self, prompt: &str) -> Result<ValidationResult, EngineError> {
        let normalized = normalize_for_detection(prompt);

        if self
            .allow_keywords
            .iter()
            .any(|keyword| !keyword.is_empty() && normalized.contains(keyword))
        {
            return Ok(ValidationResult {
                is_safe: true,
                reason: "BYPASS_ALLOW_KEYWORD".to_string(),
                score: 0,
            });
        }

        if let Some(matched) = self.deny_keyword_automaton.find(&normalized) {
            let reason = self
                .deny_keywords
                .get(matched.pattern().as_usize())
                .map(|keyword| format!("BLOCK_DENY_KEYWORD:{keyword}"))
                .unwrap_or_else(|| "BLOCK_DENY_KEYWORD".to_string());

            return Ok(ValidationResult {
                is_safe: false,
                reason,
                score: self.sensitivity_score,
            });
        }

        if self
            .deny_patterns
            .iter()
            .any(|pattern| pattern.is_match(&normalized))
        {
            return Ok(ValidationResult {
                is_safe: false,
                reason: "BLOCK_DENY_PATTERN".to_string(),
                score: self.sensitivity_score,
            });
        }

        Ok(ValidationResult {
            is_safe: true,
            reason: "PASS".to_string(),
            score: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::filter_pack::models::FilterPackSettings;
    use crate::filter_pack::FilterPack;

    use super::{EngineError, FilterEngine, ValidationResult};

    fn build_test_engine() -> Result<FilterEngine, EngineError> {
        let pack = FilterPack {
            version: "1.0.0".to_string(),
            last_updated: "2026-02-22".to_string(),
            deny_keywords: vec!["시스템".to_string(), "ignore".to_string()],
            deny_patterns: vec!["시스템.*지침".to_string()],
            allow_keywords: vec!["internal-approved-test".to_string()],
            settings: FilterPackSettings {
                sensitivity_score: 70,
            },
        };

        FilterEngine::new(&pack)
    }

    fn validate_case(prompt: &str) -> Result<ValidationResult, EngineError> {
        build_test_engine().and_then(|engine| engine.validate(prompt))
    }

    #[test]
    fn blocks_korean_nfd_jamo_attack() {
        let result = validate_case("ㅅㅣㅅㅡㅌㅔㅁ");

        assert!(matches!(
            result,
            Ok(ref value) if !value.is_safe && value.reason.starts_with("BLOCK_DENY_KEYWORD")
        ));
    }

    #[test]
    fn blocks_korean_whitespace_evasion_attack() {
        let result = validate_case("시 스 템 지 침");

        assert!(matches!(
            result,
            Ok(ref value) if !value.is_safe && value.reason.starts_with("BLOCK_DENY_KEYWORD")
        ));
    }

    #[test]
    fn blocks_english_case_and_symbol_evasion_attack() {
        let result = validate_case("I.g.n.o.R.e");

        assert!(matches!(
            result,
            Ok(ref value) if !value.is_safe && value.reason.starts_with("BLOCK_DENY_KEYWORD")
        ));
    }

    #[test]
    fn bypasses_when_allow_keyword_matches_first() {
        let result = validate_case("internal-approved-test ignore");

        assert!(matches!(
            result,
            Ok(ref value) if value.is_safe && value.reason == "BYPASS_ALLOW_KEYWORD"
        ));
    }
}
