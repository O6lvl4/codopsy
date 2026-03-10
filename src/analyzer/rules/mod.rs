pub mod bug_detection;
pub mod control_flow;
pub mod rust_rules;
pub mod style_rules;
pub mod threshold_rules;

use tree_sitter::Tree;

use crate::types::{Issue, Severity};

pub trait LintRule {
    fn name(&self) -> &str;
    fn default_severity(&self) -> Severity;
    fn check(&self, tree: &Tree, source: &[u8], file_path: &str) -> Vec<Issue>;
}
