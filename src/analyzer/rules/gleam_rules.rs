use tree_sitter::Tree;

use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `todo` expressions (placeholder that crashes at runtime).
pub fn check_no_todo(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "todo" {
            ctx.report(node, "no-todo", "Unexpected `todo` expression; implement before shipping".into());
        }
    })
}

/// Detect `panic` expressions (intentional crash).
pub fn check_no_panic(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "panic" {
            ctx.report(node, "no-panic", "Avoid `panic`; return a `Result` instead".into());
        }
    })
}

/// Detect `let assert` patterns (can crash at runtime on mismatch).
pub fn check_no_let_assert(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "let_assert" && node.kind() != "assert" {
            return;
        }
        // In Gleam's tree-sitter, `let assert` is represented as its own node kind
        ctx.report(
            node,
            "no-let-assert",
            "Avoid `let assert`; it crashes on pattern mismatch, use `case` instead".into(),
        );
    })
}
