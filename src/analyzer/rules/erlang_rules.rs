use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `process_flag(trap_exit, true)` calls.
pub fn check_no_process_flag(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        // In Erlang tree-sitter, function calls have a function name child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "atom" && node_text(&child, ctx.source) == "process_flag" {
                ctx.report(
                    node,
                    "no-process-flag",
                    "Avoid `process_flag(trap_exit, true)`; use supervisors for fault tolerance".into(),
                );
                return;
            }
        }
    })
}

/// Detect catch-all pattern as the first clause in case/try expressions.
pub fn check_no_catch_all(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "case_expr" {
            return;
        }
        let mut cursor = node.walk();
        let clauses: Vec<_> = node.children(&mut cursor)
            .filter(|c| c.kind() == "cr_clause")
            .collect();
        if clauses.is_empty() {
            return;
        }
        // Check if the first clause uses a wildcard/variable (catch-all)
        let first = &clauses[0];
        let mut first_cursor = first.walk();
        for child in first.children(&mut first_cursor) {
            if child.kind() == "variable" && node_text(&child, ctx.source) == "_" {
                ctx.report(
                    first,
                    "no-catch-all",
                    "Catch-all `_` as first clause; place specific patterns before catch-all".into(),
                );
                return;
            }
        }
    })
}

/// Detect `exit()` calls.
pub fn check_no_exit_call(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "atom" && node_text(&child, ctx.source) == "exit" {
                ctx.report(
                    node,
                    "no-exit-call",
                    "Avoid `exit()`; let supervisors handle process termination".into(),
                );
                return;
            }
        }
    })
}
