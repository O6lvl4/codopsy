use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect global variable assignments (missing `local`).
/// Inspired by Luacheck W111/W112.
pub fn check_no_global(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "variable_assignment" {
            return;
        }
        // If there's no `local` keyword before the assignment, it's global
        // Check parent context — top-level assignments without `local` are global
        if let Some(parent) = node.parent() {
            if parent.kind() == "chunk" || parent.kind() == "block" {
                // Check if this is a local_variable_declaration instead
                // variable_assignment at top level = global
                let text = node_text(node, ctx.source);
                if !text.starts_with("local ") {
                    let Some(names) = node.child(0) else { return };
                    let var_name = node_text(&names, ctx.source);
                    // Skip common globals
                    if !matches!(var_name, "_G" | "_ENV" | "arg" | "_VERSION") {
                        ctx.report(node, "no-global", format!("Global variable `{var_name}`; use `local`"));
                    }
                }
            }
        }
    })
}

/// Detect `os.execute` / `io.popen` (shell injection risk).
pub fn check_no_os_execute(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("os.execute") || text.starts_with("io.popen") {
            ctx.report(node, "no-os-execute", "Avoid `os.execute`/`io.popen`; shell injection risk".into());
        }
    })
}

/// Detect `loadstring` / `load` with string argument (code injection).
pub fn check_no_loadstring(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call" {
            return;
        }
        let text = node_text(node, ctx.source);
        if text.starts_with("loadstring(") || text.starts_with("load(") {
            ctx.report(node, "no-loadstring", "Avoid `loadstring`/`load` with string; code injection risk".into());
        }
    })
}

/// Detect `print` calls (use logging framework).
pub fn check_no_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_call" {
            return;
        }
        let Some(func) = node.child(0) else { return };
        if node_text(&func, ctx.source) == "print" {
            ctx.report(node, "no-print", "Avoid `print()`; use a logging framework".into());
        }
    })
}
