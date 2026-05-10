use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `print()` calls (use os_log or Logger).
/// Inspired by SwiftLint's no_print.
pub fn check_no_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "print" | "debugPrint" | "dump") {
            ctx.report(node, "no-print", format!("Avoid `{name}()`; use `os_log` or `Logger`"));
        }
    })
}

/// Detect force unwrapping `!` on optionals.
/// Inspired by SwiftLint's force_unwrapping.
pub fn check_no_force_unwrap(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "force_unwrap_expression" {
            ctx.report(node, "no-force-unwrap", "Avoid force unwrapping `!`; use `if let` or `guard let`".into());
        }
    })
}

/// Detect force try `try!`.
/// Inspired by SwiftLint's force_try.
pub fn check_no_force_try(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "try_expression" {
            let text = node_text(node, ctx.source);
            if text.starts_with("try!") {
                ctx.report(node, "no-force-try", "Avoid `try!`; use `do/catch` or `try?`".into());
            }
        }
    })
}

/// Detect force cast `as!`.
/// Inspired by SwiftLint's force_cast.
pub fn check_no_force_cast(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "as_expression" {
            let text = node_text(node, ctx.source);
            if text.contains("as!") {
                ctx.report(node, "no-force-cast", "Avoid `as!` force cast; use `as?` with optional binding".into());
            }
        }
    })
}

/// Detect `NSLog` (use os_log instead).
pub fn check_no_nslog(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        if node_text(&func, ctx.source) == "NSLog" {
            ctx.report(node, "no-nslog", "Avoid `NSLog`; use `os_log` or `Logger`".into());
        }
    })
}

/// Detect `fatalError()` outside of precondition contexts.
pub fn check_no_fatal_error(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        if node_text(&func, ctx.source) == "fatalError" {
            ctx.report(node, "no-fatal-error", "Avoid `fatalError()`; handle errors gracefully".into());
        }
    })
}
