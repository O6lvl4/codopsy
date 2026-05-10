use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `head` / `tail` / `init` / `last` on lists (partial functions, crash on empty).
/// Inspired by HLint.
pub fn check_no_partial_functions(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "variable" && node.kind() != "function_application" {
            return;
        }
        let text = node_text(node, ctx.source);
        let partials = ["head", "tail", "init", "last", "fromJust", "read"];
        for p in &partials {
            if text == *p || text.starts_with(&format!("{p} ")) {
                ctx.report(node, "no-partial-function", format!("`{p}` is partial; crashes on empty input. Use pattern matching or safe alternatives"));
                return;
            }
        }
    })
}

/// Detect `undefined` (indicates unfinished code).
pub fn check_no_undefined(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "variable" && node_text(node, ctx.source) == "undefined" {
            ctx.report(node, "no-undefined", "`undefined` will crash at runtime; implement the function".into());
        }
    })
}

/// Detect `error` calls (throws exception).
pub fn check_no_error(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "variable" && node_text(node, ctx.source) == "error" {
            if let Some(parent) = node.parent() {
                if parent.kind() == "function_application" {
                    ctx.report(node, "no-error", "`error` throws an exception; return `Maybe`/`Either` instead".into());
                }
            }
        }
    })
}

/// Detect `unsafePerformIO` usage.
pub fn check_no_unsafe_perform_io(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "variable" && node_text(node, ctx.source) == "unsafePerformIO" {
            ctx.report(node, "no-unsafe-perform-io", "`unsafePerformIO` breaks referential transparency".into());
        }
    })
}

/// Detect `Debug.trace` (debug output left in code).
pub fn check_no_trace(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let text = node_text(node, ctx.source);
        if matches!(text, "trace" | "traceShow" | "traceShowId") && node.kind() == "variable" {
            ctx.report(node, "no-trace", format!("Remove debug `{text}` call"));
        }
    })
}
