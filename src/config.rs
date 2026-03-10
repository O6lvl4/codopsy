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
        let config_path = dir.join(CONFIG_FILENAME);
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<CodopsyConfig>(&content) {
                    return config;
                }
            }
            return CodopsyConfig::default();
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
    if let Some(home) = home {
        let home_config = home.join(CONFIG_FILENAME);
        if home_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&home_config) {
                if let Ok(config) = serde_json::from_str::<CodopsyConfig>(&content) {
                    return config;
                }
            }
        }
    }

    CodopsyConfig::default()
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
