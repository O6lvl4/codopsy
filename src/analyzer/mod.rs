pub mod ast_utils;
pub mod complexity;
pub mod linter;
mod node_classify;
pub mod rules;

use crate::config::CodopsyConfig;
use crate::types::{ComplexityResult, FileAnalysis, Issue};

pub fn analyze_file(file_path: &str, config: &CodopsyConfig) -> FileAnalysis {
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: ComplexityResult {
                    cyclomatic: 0,
                    cognitive: 0,
                    functions: vec![],
                },
                issues: vec![Issue {
                    file: file_path.to_string(),
                    line: 1,
                    column: 1,
                    severity: crate::types::Severity::Error,
                    rule: "parse-error".to_string(),
                    message: format!("Failed to read file: {e}"),
                }],
                score: None,
            };
        }
    };

    let language = match ast_utils::get_language(file_path) {
        Some(l) => l,
        None => {
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: ComplexityResult {
                    cyclomatic: 0,
                    cognitive: 0,
                    functions: vec![],
                },
                issues: vec![],
                score: None,
            };
        }
    };
    let tree = match ast_utils::parse_source(&source, language) {
        Some(t) => t,
        None => {
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: ComplexityResult {
                    cyclomatic: 0,
                    cognitive: 0,
                    functions: vec![],
                },
                issues: vec![Issue {
                    file: file_path.to_string(),
                    line: 1,
                    column: 1,
                    severity: crate::types::Severity::Error,
                    rule: "parse-error".to_string(),
                    message: "Failed to parse file".to_string(),
                }],
                score: None,
            };
        }
    };

    let complexity = complexity::analyze_complexity(&tree, source.as_bytes());
    let issues = linter::lint_file(file_path, &source, &tree, config, language);

    FileAnalysis {
        file: file_path.to_string(),
        complexity,
        issues,
        score: None,
    }
}
