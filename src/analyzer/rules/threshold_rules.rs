use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{is_function_node, node_column, node_line, SourceLanguage};
use crate::types::{Issue, Severity};

/// max-lines: Warn if a file exceeds a maximum number of lines.
pub fn check_max_lines(
    _tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let line_count = source.iter().filter(|&&b| b == b'\n').count() + 1;
    if line_count > max {
        vec![Issue {
            file: file_path.to_string(),
            line: 1,
            column: 1,
            severity,
            rule: "max-lines".to_string(),
            message: format!("File has {line_count} lines (max: {max})"),
        }]
    } else {
        vec![]
    }
}

fn is_depth_increasing_js(kind: &str) -> bool {
    matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
    )
}

fn is_depth_increasing_rust(kind: &str) -> bool {
    matches!(
        kind,
        "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    )
}

/// max-depth with language awareness.
pub fn check_max_depth_for_language(
    tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
    language: SourceLanguage,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    fn walk(
        node: &Node,
        depth: usize,
        file_path: &str,
        severity: Severity,
        max: usize,
        is_rust: bool,
        issues: &mut Vec<Issue>,
    ) {
        let kind = node.kind();
        let increases = if is_rust {
            is_depth_increasing_rust(kind)
        } else {
            is_depth_increasing_js(kind)
        };

        let new_depth = if increases { depth + 1 } else { depth };

        if increases && new_depth > max {
            issues.push(Issue {
                file: file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity,
                rule: "max-depth".to_string(),
                message: format!("Nesting depth {new_depth} exceeds max of {max}"),
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(&child, new_depth, file_path, severity, max, is_rust, issues);
        }
    }

    let _ = source;
    walk(
        &tree.root_node(),
        0,
        file_path,
        severity,
        max,
        language.is_rust(),
        &mut issues,
    );
    issues
}

/// max-depth: JS/TS version (backward compat).
pub fn check_max_depth(
    tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    check_max_depth_for_language(tree, source, file_path, severity, max, SourceLanguage::JavaScript)
}

/// max-params: Warn if a function has too many parameters.
/// Works for both JS/TS and Rust.
pub fn check_max_params(
    tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    fn visit(
        node: &Node,
        source: &[u8],
        file_path: &str,
        severity: Severity,
        max: usize,
        issues: &mut Vec<Issue>,
    ) {
        if is_function_node(node) {
            if let Some(params) = node.child_by_field_name("parameters") {
                let count = count_params(&params);
                if count > max {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "max-params".to_string(),
                        message: format!("Function has {count} parameters (max: {max})"),
                    });
                }
            }
        }

        let _ = source;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, max, issues);
        }
    }

    visit(&tree.root_node(), source, file_path, severity, max, &mut issues);
    issues
}

fn count_params(params: &Node) -> usize {
    let mut count = 0;
    let mut cursor = params.walk();
    for child in params.children(&mut cursor) {
        let k = child.kind();
        // Skip delimiters and non-parameter nodes
        if k == "("
            || k == ")"
            || k == ","
            || k == "|"       // Rust closure delimiters
            || k == "comment"
            || k == "line_comment"
            || k == "block_comment"
        {
            continue;
        }
        // Rust: self_parameter counts as a parameter
        count += 1;
    }
    count
}
