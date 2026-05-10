use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `println` / `print` calls.
/// Inspired by Detekt's ForbiddenMethodCall.
pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "println" | "print") {
            ctx.report(node, "no-println", format!("Avoid `{name}()`; use a logging framework"));
        }
    })
}

/// Detect force cast `as` (unsafe, throws ClassCastException).
/// Inspired by Detekt's UnsafeCast.
pub fn check_no_unsafe_cast(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "as_expression" {
            let text = node_text(node, ctx.source);
            // `as` (not `as?`) is unsafe
            if text.contains(" as ") && !text.contains(" as? ") {
                ctx.report(node, "no-unsafe-cast", "Avoid unsafe `as` cast; use `as?` with null check".into());
            }
        }
    })
}

/// Detect `!!` (not-null assertion, throws NullPointerException).
/// Inspired by Detekt's UnnecessaryNotNullOperator.
pub fn check_no_not_null_assertion(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "postfix_expression" || node.kind() == "not_null_expression" {
            let text = node_text(node, ctx.source);
            if text.ends_with("!!") {
                ctx.report(node, "no-not-null-assertion", "Avoid `!!`; use safe calls `?.` or `let`".into());
            }
        }
    })
}

/// Detect empty catch blocks.
pub fn check_no_empty_catch(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "catch_block" {
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "statements" {
                let mut inner = child.walk();
                let has_stmt = child.children(&mut inner).any(|c| {
                    !matches!(c.kind(), "{" | "}" | "comment" | "multiline_comment")
                });
                if !has_stmt {
                    ctx.report(node, "no-empty-catch", "Empty catch block; handle or rethrow the exception".into());
                }
            }
        }
    })
}

/// Detect `System.exit()` calls.
pub fn check_no_system_exit(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.contains("System.exit") || text.contains("exitProcess") {
            ctx.report(node, "no-system-exit", "Avoid `System.exit()`/`exitProcess()`; throw an exception instead".into());
        }
    })
}

/// Detect `var` that could be `val` (mutable where immutable suffices).
/// Simplified: just flag all `var` at property level as a hint.
pub fn check_prefer_val(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "property_declaration" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("var ") {
            ctx.report(node, "prefer-val", "Consider using `val` instead of `var` for immutability".into());
        }
    })
}
