use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Get the first `sym_lit` child of a `list_lit` node (the function name in a Clojure call).
fn first_sym(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "sym_lit" {
            return Some(node_text(&child, source).to_string());
        }
    }
    None
}

/// Detect `(println ...)` calls (debug output).
pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "list_lit" {
            return;
        }
        if let Some(name) = first_sym(node, ctx.source) {
            if matches!(name.as_str(), "println" | "prn" | "print") {
                ctx.report(
                    node,
                    "no-println",
                    format!("Avoid `({name} ...)`, use a logging library"),
                );
            }
        }
    })
}

/// Detect nested `def`/`defn` inside `defn` (anti-pattern).
pub fn check_no_def_in_def(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "list_lit" {
            return;
        }
        let Some(name) = first_sym(node, ctx.source) else { return };
        if !matches!(name.as_str(), "def" | "defn" | "defn-") {
            return;
        }
        // Walk up to check if inside another defn
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "list_lit" {
                if let Some(parent_name) = first_sym(&parent, ctx.source) {
                    if matches!(parent_name.as_str(), "defn" | "defn-") {
                        ctx.report(
                            node,
                            "no-def-in-def",
                            format!("Avoid `{name}` inside `{parent_name}`; use `let` or `letfn` for local bindings"),
                        );
                        return;
                    }
                }
            }
            current = parent.parent();
        }
    })
}

/// Detect `(Thread/sleep ...)` calls.
pub fn check_no_thread_sleep(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "list_lit" {
            return;
        }
        if let Some(name) = first_sym(node, ctx.source) {
            if name == "Thread/sleep" {
                ctx.report(
                    node,
                    "no-thread-sleep",
                    "Avoid `Thread/sleep`; use async scheduling or core.async timeouts".into(),
                );
            }
        }
    })
}

/// Detect Java reflection calls like `(.getClass obj)` (dot-method interop).
pub fn check_no_reflection(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "list_lit" {
            return;
        }
        if let Some(name) = first_sym(node, ctx.source) {
            if name.starts_with('.') && name.len() > 1 && name != ".." {
                ctx.report(
                    node,
                    "no-reflection",
                    format!("Java reflection call `{name}` detected; prefer protocol/type-hinted access"),
                );
            }
        }
    })
}
