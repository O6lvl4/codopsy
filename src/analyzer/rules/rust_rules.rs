use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line, node_text};
use crate::types::{Issue, Severity};

/// no-unsafe: Warn on `unsafe` blocks.
pub fn check_no_unsafe(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "unsafe_block" {
            issues.push(Issue {
                file: file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity,
                rule: "no-unsafe".to_string(),
                message: "Avoid `unsafe` block".to_string(),
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

/// no-unwrap: Warn on `.unwrap()` calls.
pub fn check_no_unwrap(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                if func.kind() == "field_expression" {
                    if let Some(field) = func.child_by_field_name("field") {
                        let name = node_text(&field, source);
                        if name == "unwrap" {
                            issues.push(Issue {
                                file: file_path.to_string(),
                                line: node_line(node),
                                column: node_column(node),
                                severity,
                                rule: "no-unwrap".to_string(),
                                message: "Avoid `.unwrap()`, use `?` or handle the error explicitly".to_string(),
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

/// no-dbg: Warn on `dbg!()` macro calls.
pub fn check_no_dbg(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "macro_invocation" {
            if let Some(macro_node) = node.child_by_field_name("macro") {
                let name = node_text(&macro_node, source);
                if name == "dbg" {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "no-dbg".to_string(),
                        message: "Unexpected `dbg!()` macro".to_string(),
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

/// no-todo: Warn on `todo!()` and `unimplemented!()` macros.
pub fn check_no_todo(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "macro_invocation" {
            if let Some(macro_node) = node.child_by_field_name("macro") {
                let name = node_text(&macro_node, source);
                if name == "todo" || name == "unimplemented" {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "no-todo".to_string(),
                        message: format!("Unexpected `{name}!()` macro"),
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

/// no-println: Warn on `println!()` / `print!()` / `eprintln!()` / `eprint!()` macros.
pub fn check_no_println(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "macro_invocation" {
            if let Some(macro_node) = node.child_by_field_name("macro") {
                let name = node_text(&macro_node, source);
                if matches!(name, "println" | "print" | "eprintln" | "eprint") {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity,
                        rule: "no-println".to_string(),
                        message: format!("Unexpected `{name}!()` macro, use a logging framework"),
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

/// no-empty-function: Warn on empty function bodies (Rust version).
pub fn check_no_empty_function_rust(tree: &Tree, source: &[u8], file_path: &str, severity: Severity) -> Vec<Issue> {
    let mut issues = Vec::new();
    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "function_item" {
            if let Some(body) = node.child_by_field_name("body") {
                if body.kind() == "block" {
                    let mut has_statements = false;
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        let k = child.kind();
                        if k != "{" && k != "}" && k != "line_comment" && k != "block_comment" {
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
        let _ = source;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }
    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}
