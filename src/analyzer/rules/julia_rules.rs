use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `println` / `print` calls (prefer @info/@warn from Logging).
pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "println" | "print" | "dump") {
            ctx.report(node, "no-println", format!("Avoid `{name}()`; use `@info`/`@warn` from Logging"));
        }
    })
}

/// Detect `eval` calls.
pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        if node_text(&func, ctx.source) == "eval" {
            ctx.report(node, "no-eval", "`eval` is a security risk and performance bottleneck".into());
        }
    })
}

/// Detect global variables (non-const, non-function at module scope).
pub fn check_no_global_mutable(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "assignment" {
            return;
        }
        // Top-level assignment (parent is source_file or module_definition)
        if let Some(parent) = node.parent() {
            if matches!(parent.kind(), "source_file" | "module_definition") {
                let text = node_text(node, ctx.source);
                if !text.starts_with("const ") {
                    ctx.report(node, "no-global-mutable", "Avoid mutable global variables; use `const` or pass as arguments".into());
                }
            }
        }
    })
}

/// Detect `ccall` without proper error handling.
pub fn check_no_bare_ccall(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        if node_text(&func, ctx.source) == "ccall" {
            ctx.report(node, "no-bare-ccall", "`ccall` requires careful error handling; consider using a wrapper".into());
        }
    })
}
