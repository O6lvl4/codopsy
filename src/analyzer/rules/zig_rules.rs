use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `@import("std").debug.print` (debug prints).
pub fn check_no_debug_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.contains("std.debug.print") || text.contains("std.log") {
            ctx.report(node, "no-debug-print", "Avoid `std.debug.print` in production code".into());
        }
    })
}

/// Detect `unreachable` (indicates unfinished logic or potential bug).
pub fn check_no_unreachable(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "identifier" && node_text(node, ctx.source) == "unreachable" {
            // Check if it's used as an expression (not a type or field name)
            if let Some(parent) = node.parent() {
                if matches!(parent.kind(), "call_expression" | "block" | "expression_statement") {
                    ctx.report(node, "no-unreachable", "`unreachable` indicates unfinished logic".into());
                }
            }
        }
    })
}

/// Detect `@panic` calls.
pub fn check_no_panic(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "builtin_call_expression" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("@panic") {
            ctx.report(node, "no-panic", "Avoid `@panic`; return an error instead".into());
        }
    })
}

/// Detect catch-all `_ =>` in switch (may hide unhandled cases).
pub fn check_no_catch_all_switch(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_expression" {
            return;
        }
        let mut cursor = node.walk();
        let prongs: Vec<_> = node.children(&mut cursor)
            .filter(|c| c.kind() == "switch_prong")
            .collect();
        // Check if last prong is a catch-all
        if let Some(last) = prongs.last() {
            let text = node_text(last, ctx.source);
            if text.starts_with("else") || text.trim_start().starts_with("_ =>") {
                // Only flag if there's more than one prong
                if prongs.len() > 1 {
                    ctx.report(last, "no-catch-all-switch", "Catch-all `else`/`_` in switch may hide unhandled cases".into());
                }
            }
        }
    })
}
