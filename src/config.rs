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
pub struct CodopsyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<HashMap<String, RuleConfig>>,
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
    serde_json::from_str(&content).ok()
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
