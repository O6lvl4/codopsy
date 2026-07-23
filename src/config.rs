use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::Severity;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    Disabled(bool),
    Severity(Severity),
    Options {
        #[serde(skip_serializing_if = "Option::is_none")]
        severity: Option<Severity>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        props: Option<bool>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodopsyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<HashMap<String, RuleConfig>>,
    /// Additional directory patterns to skip (merged with built-in defaults).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_dirs: Option<Vec<String>>,
    /// Additional file patterns to skip (merged with built-in defaults).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_files: Option<Vec<String>>,
}

impl CodopsyConfig {
    pub fn is_rule_disabled(&self, name: &str) -> bool {
        if let Some(rules) = &self.rules {
            if let Some(RuleConfig::Disabled(false)) = rules.get(name) {
                return true;
            }
        }
        false
    }

    pub fn get_rule_severity(&self, name: &str) -> Option<Severity> {
        if let Some(rules) = &self.rules {
            match rules.get(name) {
                Some(RuleConfig::Severity(s)) => Some(*s),
                Some(RuleConfig::Options {
                    severity: Some(s), ..
                }) => Some(*s),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_rule_max(&self, name: &str) -> Option<usize> {
        if let Some(rules) = &self.rules {
            if let Some(RuleConfig::Options { max: Some(m), .. }) = rules.get(name) {
                return Some(*m);
            }
        }
        None
    }

    /// Resolve the effective threshold for a `max`-style rule: the config's
    /// override if present, else `default_max`, or `None` if the rule is
    /// disabled entirely. This is the single source of truth both for
    /// deciding when to emit a `rule` issue and for computing its
    /// contribution to `score_complexity` — the two must never diverge, or a
    /// relaxed config threshold silently keeps penalizing the score.
    pub fn resolve_threshold(&self, name: &str, default_max: usize) -> Option<usize> {
        if self.is_rule_disabled(name) {
            return None;
        }
        Some(self.get_rule_max(name).unwrap_or(default_max))
    }
}

const CONFIG_FILENAME: &str = ".codopsyrc.json";

pub fn load_config(target_dir: &Path) -> CodopsyConfig {
    let mut dir = target_dir.to_path_buf();
    let home = dirs_home();

    loop {
        if let Some(config) = try_load_from(&dir) {
            return config;
        }
        if Some(&dir) == home.as_ref() {
            break;
        }
        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => break,
        }
    }

    // Check home directory as last resort
    home.and_then(|h| try_load_from(&h)).unwrap_or_default()
}

fn try_load_from(dir: &Path) -> Option<CodopsyConfig> {
    let path = dir.join(CONFIG_FILENAME);
    let content = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str(&content) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!(
                "Warning: invalid config in {}: {}",
                path.display(),
                e
            );
            None
        }
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    #[test]
    fn default_config_has_no_rules() {
        let config = CodopsyConfig::default();
        assert!(config.rules.is_none());
        assert!(!config.is_rule_disabled("no-any"));
        assert!(config.get_rule_severity("no-any").is_none());
        assert!(config.get_rule_max("max-lines").is_none());
    }

    #[test]
    fn parse_severity_rule() {
        let json = r#"{ "rules": { "no-any": "error" } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.get_rule_severity("no-any"), Some(Severity::Error));
        assert!(!config.is_rule_disabled("no-any"));
    }

    #[test]
    fn parse_disabled_rule() {
        let json = r#"{ "rules": { "no-console": false } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert!(config.is_rule_disabled("no-console"));
    }

    #[test]
    fn parse_options_rule() {
        let json = r#"{ "rules": { "max-lines": { "severity": "warning", "max": 500 } } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.get_rule_severity("max-lines"), Some(Severity::Warning));
        assert_eq!(config.get_rule_max("max-lines"), Some(500));
    }

    #[test]
    fn parse_skip_dirs() {
        let json = r#"{ "skipDirs": ["/custom/"] }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.skip_dirs.as_ref().unwrap(), &["/custom/"]);
    }

    #[test]
    fn unknown_rule_returns_none() {
        let json = r#"{ "rules": {} }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert!(config.get_rule_severity("nonexistent").is_none());
        assert!(config.get_rule_max("nonexistent").is_none());
        assert!(!config.is_rule_disabled("nonexistent"));
    }

    #[test]
    fn resolve_threshold_falls_back_to_default() {
        let config = CodopsyConfig::default();
        assert_eq!(config.resolve_threshold("max-complexity", 10), Some(10));
    }

    #[test]
    fn resolve_threshold_uses_configured_max() {
        let json = r#"{ "rules": { "max-complexity": { "max": 20 } } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.resolve_threshold("max-complexity", 10), Some(20));
    }

    #[test]
    fn resolve_threshold_none_when_disabled() {
        let json = r#"{ "rules": { "max-complexity": false } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.resolve_threshold("max-complexity", 10), None);
    }
}
