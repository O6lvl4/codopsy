use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `print()` calls (use dart:developer log or a logger).
/// Inspired by dart linter's avoid_print.
pub fn check_no_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "identifier" {
            return;
        }
        if node_text(node, ctx.source) != "print" {
            return;
        }
        if let Some(parent) = node.parent() {
            if parent.kind() == "selector" || parent.kind() == "function_expression_body" {
                return;
            }
        }
        ctx.report(node, "no-print", "Avoid `print()`; use a logging package".into());
    })
}

/// Detect `dynamic` type annotations.
/// Inspired by dart linter's avoid_annotating_with_dynamic.
pub fn check_no_dynamic(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "type_identifier" && node_text(node, ctx.source) == "dynamic" {
            ctx.report(node, "no-dynamic", "Avoid `dynamic` type; use specific types".into());
        }
    })
}

/// Detect empty catch blocks.
pub fn check_no_empty_catch(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "catch_clause" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let has_statement = body.children(&mut cursor).any(|c| {
            !matches!(c.kind(), "{" | "}" | "comment")
        });
        if !has_statement {
            ctx.report(node, "no-empty-catch", "Empty catch block; handle or rethrow the exception".into());
        }
    })
}

/// Detect `as` casts (prefer `is` check or pattern matching).
pub fn check_no_cast(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "as_expression" {
            ctx.report(node, "no-cast", "Avoid `as` cast; use `is` type check or pattern matching".into());
        }
    })
}

/// Detect `rethrow` in unnecessary try/catch.
pub fn check_no_rethrow_only(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "catch_clause" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "{" | "}" | "comment"))
            .collect();
        if stmts.len() == 1 && node_text(&stmts[0], ctx.source).trim() == "rethrow;" {
            ctx.report(node, "no-rethrow-only", "Catch block only rethrows; remove the try/catch".into());
        }
    })
}
