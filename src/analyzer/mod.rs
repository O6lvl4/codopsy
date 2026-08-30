pub mod ast_utils;
pub mod complexity;
pub mod decl;
pub mod language;
pub mod linter;
mod node_classify;
mod rule_registry;
mod rule_tables;
pub mod rules;

use crate::config::CodopsyConfig;
use crate::defaults;
use crate::types::{ComplexityResult, FileAnalysis, Issue};

fn empty_complexity() -> ComplexityResult {
    ComplexityResult {
        cyclomatic: 0,
        cognitive: 0,
        functions: vec![],
    }
}

pub fn analyze_file(file_path: &str, config: &CodopsyConfig) -> FileAnalysis {
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: empty_complexity(),
                issues: vec![Issue {
                    file: file_path.to_string(),
                    line: 1,
                    column: 1,
                    severity: crate::types::Severity::Error,
                    rule: "parse-error".to_string(),
                    message: format!("Failed to read file: {e}"),
                }],
                score: None,
                unanalyzed: false,
            };
        }
    };

    let language = match ast_utils::get_language(file_path) {
        Some(l) => l,
        None => {
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: empty_complexity(),
                issues: vec![],
                score: None,
                unanalyzed: false,
            };
        }
    };
    let tree = match ast_utils::parse_source(&source, language) {
        Some(t) => t,
        None => {
            // The grammar could not build a tree at all — nothing to analyze.
            return FileAnalysis {
                file: file_path.to_string(),
                complexity: empty_complexity(),
                issues: vec![Issue {
                    file: file_path.to_string(),
                    line: 1,
                    column: 1,
                    severity: crate::types::Severity::Error,
                    rule: "parse-error".to_string(),
                    message: "Failed to parse file".to_string(),
                }],
                score: None,
                unanalyzed: true,
            };
        }
    };

    let complexity = complexity::analyze_complexity(&tree, source.as_bytes(), language);
    let issues = linter::lint_file(file_path, &source, &tree, config, language);

    // A file the grammar could only partially read yields almost no functions
    // and almost no issues, which would otherwise score near-perfect. When the
    // unparsed share is large, treat it as unanalyzed: keep whatever we found
    // (the `syntax-error` issue included) but leave it unscored so it is
    // excluded from the project score instead of masquerading as clean.
    let unanalyzed = rules::universal_rules::scan_parse_coverage(&tree)
        .is_some_and(|c| c.unparsed_share(source.len()) >= defaults::UNANALYZED_MIN_SHARE);

    FileAnalysis {
        file: file_path.to_string(),
        complexity,
        issues,
        score: None,
        unanalyzed,
    }
}
