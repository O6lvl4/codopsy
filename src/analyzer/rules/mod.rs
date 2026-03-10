pub mod bug_detection;
pub mod control_flow;
pub mod rust_rules;
pub mod style_rules;
pub mod threshold_rules;

use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line};
use crate::types::{Issue, Severity};

/// Shared context for lint rule checks, reducing parameter count.
pub struct Ctx<'a> {
    pub source: &'a [u8],
    pub file_path: &'a str,
    pub severity: Severity,
    pub issues: Vec<Issue>,
}

impl<'a> Ctx<'a> {
    pub fn new(source: &'a [u8], file_path: &'a str, severity: Severity) -> Self {
        Self { source, file_path, severity, issues: Vec::new() }
    }

    pub fn report(&mut self, node: &Node, rule: &str, message: String) {
        self.issues.push(Issue {
            file: self.file_path.to_string(),
            line: node_line(node),
            column: node_column(node),
            severity: self.severity,
            rule: rule.to_string(),
            message,
        });
    }
}

/// Helper: run a visitor function over the tree and return collected issues.
pub fn run_check(
    tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    visitor: fn(&Node, &mut Ctx),
) -> Vec<Issue> {
    let mut ctx = Ctx::new(source, file_path, severity);
    visit_all(&tree.root_node(), &mut ctx, visitor);
    ctx.issues
}

fn visit_all(node: &Node, ctx: &mut Ctx, visitor: fn(&Node, &mut Ctx)) {
    visitor(node, ctx);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_all(&child, ctx, visitor);
    }
}
