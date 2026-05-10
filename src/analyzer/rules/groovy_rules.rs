use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `println` / `print` calls.
pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "method_call" && node.kind() != "function_call" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("println") || text.starts_with("print(") {
            ctx.report(node, "no-println", "Avoid `println`; use a logging framework".into());
        }
    })
}

/// Detect `def` without type (dynamic typing).
pub fn check_no_def_type(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "variable_declaration" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("def ") && !text.contains(":") {
            ctx.report(node, "no-def-type", "Avoid `def` without type; use explicit types for clarity".into());
        }
    })
}

/// Detect `System.exit()` calls.
pub fn check_no_system_exit(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let text = node_text(node, ctx.source);
        if text.contains("System.exit") {
            ctx.report(node, "no-system-exit", "Avoid `System.exit()`; throw an exception instead".into());
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
            ctx.report(node, "no-empty-catch", "Empty catch block".into());
        }
    })
}
