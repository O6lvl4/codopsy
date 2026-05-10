use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `var_dump`, `print_r`, `echo` debug output.
pub fn check_no_debug_output(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "var_dump" | "print_r" | "debug_zval_dump" | "dd") {
            ctx.report(node, "no-debug-output", format!("Remove debug call `{name}()`"));
        }
    })
}

/// Detect `eval()` usage.
pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if node_text(&func, ctx.source) == "eval" {
            ctx.report(node, "no-eval", "`eval()` is a security risk".into());
        }
    })
}

/// Detect `die()` / `exit()` in non-entry code.
pub fn check_no_exit(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "die" | "exit") {
            ctx.report(node, "no-exit", format!("Avoid `{name}()`; throw an exception instead"));
        }
    })
}

/// Detect loose comparison `==` / `!=` (should use `===` / `!==`).
pub fn check_strict_comparison(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let text = node_text(node, ctx.source);
        // PHP binary_expression includes operator inline
        if text.contains(" == ") && !text.contains(" === ") {
            ctx.report(node, "strict-comparison", "Use `===` instead of `==` for strict comparison".into());
        }
        if text.contains(" != ") && !text.contains(" !== ") {
            ctx.report(node, "strict-comparison", "Use `!==` instead of `!=` for strict comparison".into());
        }
    })
}

/// Detect `@` error suppression operator.
pub fn check_no_error_suppression(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "error_suppression_expression" {
            ctx.report(node, "no-error-suppression", "Avoid `@` error suppression; handle errors explicitly".into());
        }
    })
}
