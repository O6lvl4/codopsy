use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `println` / `print` calls.
/// Inspired by Wartremover's NonUnitStatements and general Scala style.
pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "println" | "print" | "printf") {
            ctx.report(node, "no-println", format!("Avoid `{name}`; use a logging framework"));
        }
    })
}

/// Detect `null` usage (prefer Option).
/// Inspired by Wartremover's Null.
pub fn check_no_null(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "null_literal" {
            ctx.report(node, "no-null", "Avoid `null`; use `Option` instead".into());
        }
    })
}

/// Detect `var` declarations (prefer `val` for immutability).
/// Inspired by Wartremover's Var.
pub fn check_no_var(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "var_definition" || node.kind() == "var_declaration" {
            ctx.report(node, "no-var", "Prefer `val` over `var` for immutability".into());
        }
    })
}

/// Detect `return` keyword (idiomatic Scala uses expression-based returns).
/// Inspired by Wartremover's Return.
pub fn check_no_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "return_expression" {
            ctx.report(node, "no-return", "Avoid explicit `return`; use expression-based returns".into());
        }
    })
}

/// Detect `asInstanceOf` (unsafe cast).
/// Inspired by Wartremover's AsInstanceOf.
pub fn check_no_as_instance_of(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.contains(".asInstanceOf") {
            ctx.report(node, "no-as-instance-of", "Avoid `asInstanceOf`; use pattern matching instead".into());
        }
        if text.contains(".isInstanceOf") {
            ctx.report(node, "no-as-instance-of", "Avoid `isInstanceOf`; use pattern matching instead".into());
        }
    })
}
