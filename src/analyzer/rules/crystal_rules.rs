use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `puts`/`p`/`pp`/`print` calls (debug output).
/// Inspired by Ameba's Lint/DebuggerStatement.
pub fn check_no_puts(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(method) = node.child_by_field_name("method") else { return };
        let name = node_text(&method, ctx.source);
        if matches!(name, "puts" | "p" | "pp" | "print") && node.child_by_field_name("receiver").is_none() {
            ctx.report(node, "no-puts", format!("Avoid `{name}`; use a logger"));
        }
    })
}

/// Detect `raise` with string literal (should use specific exception class).
pub fn check_no_raise_string(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(method) = node.child_by_field_name("method") else { return };
        if node_text(&method, ctx.source) != "raise" {
            return;
        }
        let Some(args) = node.child_by_field_name("arguments") else { return };
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.kind() == "string" {
                ctx.report(
                    node,
                    "no-raise-string",
                    "Avoid `raise \"message\"`; use a specific exception class".into(),
                );
                return;
            }
        }
    })
}

/// Detect `rescue Exception` (too broad).
/// Inspired by Ameba's Lint/RescueException.
pub fn check_no_rescue_exception(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "rescue" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.contains("Exception") {
            ctx.report(
                node,
                "no-rescue-exception",
                "Avoid `rescue Exception`; rescue specific error types".into(),
            );
        }
    })
}

/// Detect `eval` / `system` / backtick commands (security risk).
pub fn check_no_shell(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(method) = node.child_by_field_name("method") else { return };
        let name = node_text(&method, ctx.source);
        if matches!(name, "system" | "exec") && node.child_by_field_name("receiver").is_none() {
            ctx.report(
                node,
                "no-shell",
                format!("Avoid `{name}`; shell commands are a security risk"),
            );
        }
    })
}

/// Detect `sleep` in non-test code.
pub fn check_no_sleep(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    if fp.contains("_spec.cr") || fp.contains("_test.cr") {
        return vec![];
    }
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(method) = node.child_by_field_name("method") else { return };
        if node_text(&method, ctx.source) == "sleep" {
            ctx.report(node, "no-sleep", "Avoid `sleep` in production code".into());
        }
    })
}
