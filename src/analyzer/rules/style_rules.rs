use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line, node_text};
use crate::types::{Issue, Severity};

/// no-any: Disallow the `any` type annotation.
pub fn check_no_any(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        // In tree-sitter-typescript, `any` is a `predefined_type` node
        if node.kind() == "predefined_type" && node_text(node, source) == "any" {
            issues.push(Issue {
                file: file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity,
                rule: "no-any".to_string(),
                message: "Unexpected `any` type".to_string(),
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}

/// no-console: Disallow `console.*` calls.
pub fn check_no_console(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "call_expression" {
            if let Some(callee) = node.child_by_field_name("function") {
                if callee.kind() == "member_expression" {
                    if let Some(obj) = callee.child_by_field_name("object") {
                        if node_text(&obj, source) == "console" {
                            issues.push(Issue {
                                file: file_path.to_string(),
                                line: node_line(node),
                                column: node_column(node),
                                severity,
                                rule: "no-console".to_string(),
                                message: "Unexpected console statement".to_string(),
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

/// no-var: Disallow `var` declarations.
pub fn check_no_var(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "variable_declaration" {
            // Check the first child which should be "var", "let", or "const"
            if let Some(first) = node.child(0) {
                if node_text(&first, source) == "var" {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "no-var".to_string(),
                        message: "Unexpected var, use let or const instead".to_string(),
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

/// eqeqeq: Require === and !== instead of == and !=.
pub fn check_eqeqeq(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "binary_expression" {
            if let Some(op) = node.child_by_field_name("operator") {
                let op_text = node_text(&op, source);
                if op_text == "==" || op_text == "!=" {
                    let suggested = if op_text == "==" { "===" } else { "!==" };
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(&op),
                        column: node_column(&op),
                        severity,
                        rule: "eqeqeq".to_string(),
                        message: format!("Expected '{suggested}' instead of '{op_text}'"),
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

/// no-empty-function: Disallow empty function bodies.
pub fn check_no_empty_function(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if crate::analyzer::ast_utils::is_function_node(node) {
            if let Some(body) = node.child_by_field_name("body") {
                if body.kind() == "statement_block" {
                    // Check if block has only whitespace/comments (no real statements)
                    let mut has_statements = false;
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if child.kind() != "{" && child.kind() != "}" && child.kind() != "comment" {
                            has_statements = true;
                            break;
                        }
                    }
                    if !has_statements {
                        issues.push(Issue {
                            file: file_path.to_string(),
                            line: node_line(node),
                            column: node_column(node),
                            severity,
                            rule: "no-empty-function".to_string(),
                            message: "Unexpected empty function".to_string(),
                        });
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

/// no-nested-ternary: Disallow nested ternary expressions.
pub fn check_no_nested_ternary(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn has_nested_ternary(node: &Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "ternary_expression" {
                return true;
            }
            if has_nested_ternary(&child) {
                return true;
            }
        }
        false
    }
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "ternary_expression" && has_nested_ternary(node) {
            issues.push(Issue {
                file: file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity,
                rule: "no-nested-ternary".to_string(),
                message: "Do not nest ternary expressions".to_string(),
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    let _ = source;
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}
