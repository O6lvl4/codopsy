use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{is_function_node, node_column, node_line};
use crate::types::{Issue, Severity};

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

fn is_depth_increment(kind: &str) -> bool {
    matches!(
        kind,
        // JS/TS / Go / Java / C / C++ / C# / PHP
        "if_statement" | "for_statement" | "for_in_statement"
            | "while_statement" | "do_statement" | "switch_statement"
            // Rust
            | "if_expression" | "for_expression" | "while_expression"
            | "loop_expression" | "match_expression"
            // Python
            | "elif_clause"
            // Ruby
            | "if" | "unless" | "for" | "while" | "until" | "case"
            // Go
            | "select_statement"
    )
}

struct DepthCtx<'a> {
    file_path: &'a str,
    severity: Severity,
    max: usize,
    issues: Vec<Issue>,
}

impl<'a> DepthCtx<'a> {
    fn walk(&mut self, node: &Node, depth: usize) {
        let kind = node.kind();
        let increases = is_depth_increment(kind);

        let new_depth = if increases { depth + 1 } else { depth };

        if increases && new_depth > self.max {
            self.issues.push(Issue {
                file: self.file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity: self.severity,
                rule: "max-depth".to_string(),
                message: format!("Nesting depth {new_depth} exceeds max of {}", self.max),
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(&child, new_depth);
        }
    }
}

pub fn check_max_depth(
    tree: &Tree,
    _source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let mut ctx = DepthCtx {
        file_path,
        severity,
        max,
        issues: Vec::new(),
    };
    ctx.walk(&tree.root_node(), 0);
    ctx.issues
}

struct ParamsCtx<'a> {
    file_path: &'a str,
    severity: Severity,
    max: usize,
    issues: Vec<Issue>,
}

impl<'a> ParamsCtx<'a> {
    fn visit(&mut self, node: &Node) {
        if is_function_node(node) {
            if let Some(params) = node.child_by_field_name("parameters") {
                let count = count_params(&params);
                if count > self.max {
                    self.issues.push(Issue {
                        file: self.file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity: self.severity,
                        rule: "max-params".to_string(),
                        message: format!("Function has {count} parameters (max: {})", self.max),
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(&child);
        }
    }
}

pub fn check_max_params(
    tree: &Tree,
    _source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let mut ctx = ParamsCtx {
        file_path,
        severity,
        max,
        issues: Vec::new(),
    };
    ctx.visit(&tree.root_node());
    ctx.issues
}

fn count_params(params: &Node) -> usize {
    let mut cursor = params.walk();
    params
        .children(&mut cursor)
        .filter(|c| {
            !matches!(c.kind(), "(" | ")" | "," | "|" | "comment" | "line_comment" | "block_comment")
        })
        .count()
}
