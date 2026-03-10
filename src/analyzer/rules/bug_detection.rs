use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line, node_text};
use crate::types::{Issue, Severity};

/// no-debugger: Disallow `debugger` statements.
pub fn check_no_debugger(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "debugger_statement" {
            issues.push(Issue {
                file: file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity,
                rule: "no-debugger".to_string(),
                message: "Unexpected 'debugger' statement".to_string(),
            });
        }
        let _ = source;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}

/// no-duplicate-case: Disallow duplicate case labels in switch.
pub fn check_no_duplicate_case(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "switch_body" {
            let mut seen = std::collections::HashSet::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "switch_case" {
                    // Get the test expression (first named child that isn't "case" keyword)
                    if let Some(value) = child.child_by_field_name("value") {
                        let text = node_text(&value, source).to_string();
                        if !seen.insert(text.clone()) {
                            issues.push(Issue {
                                file: file_path.to_string(),
                                line: node_line(&child),
                                column: node_column(&child),
                                severity,
                                rule: "no-duplicate-case".to_string(),
                                message: format!("Duplicate case label: {text}"),
                            });
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}

/// no-self-assign: Disallow `x = x`.
pub fn check_no_self_assign(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "assignment_expression" {
            if let Some(op) = node.child_by_field_name("operator") {
                if node_text(&op, source) == "=" {
                    if let (Some(left), Some(right)) = (
                        node.child_by_field_name("left"),
                        node.child_by_field_name("right"),
                    ) {
                        let left_text = node_text(&left, source);
                        let right_text = node_text(&right, source);
                        if left_text == right_text {
                            issues.push(Issue {
                                file: file_path.to_string(),
                                line: node_line(node),
                                column: node_column(node),
                                severity,
                                rule: "no-self-assign".to_string(),
                                message: format!("'{left_text}' is assigned to itself"),
                            });
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}

/// no-eval: Disallow `eval()`.
pub fn check_no_eval(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "call_expression" {
            if let Some(callee) = node.child_by_field_name("function") {
                if callee.kind() == "identifier" && node_text(&callee, source) == "eval" {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "no-eval".to_string(),
                        message: "eval() is not allowed".to_string(),
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}
