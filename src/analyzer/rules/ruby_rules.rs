use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `puts`/`p`/`pp`/`print` calls (debug output).
/// Inspired by RuboCop's Lint/Debugger.
pub fn check_no_puts(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" && node.kind() != "identifier" {
            return;
        }
        let text = if node.kind() == "call" {
            if let Some(method) = node.child_by_field_name("method") {
                node_text(&method, ctx.source)
            } else {
                return;
            }
        } else {
            // Bare identifier call like `puts "hello"`
            node_text(node, ctx.source)
        };
        if !matches!(text, "puts" | "p" | "pp" | "print") {
            return;
        }
        // Skip method calls on objects (e.g. logger.puts)
        if node.kind() == "call" && node.child_by_field_name("receiver").is_some() {
            return;
        }
        ctx.report(node, "no-puts", format!("Avoid `{text}`, use a logger"));
    })
}

/// Detect `eval` usage.
pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let method = node.child_by_field_name("method")
            .map(|m| node_text(&m, ctx.source));
        if method == Some("eval") && node.child_by_field_name("receiver").is_none() {
            ctx.report(node, "no-eval", "`eval` is a security risk; avoid dynamic code execution".into());
        }
    })
}

/// Detect `require` with relative paths that should use `require_relative`.
pub fn check_require_relative(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let method = node.child_by_field_name("method")
            .map(|m| node_text(&m, ctx.source));
        if method != Some("require") {
            return;
        }
        let Some(args) = node.child_by_field_name("arguments") else { return };
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.kind() == "string" {
                let text = node_text(&child, ctx.source);
                if text.contains("./") || text.contains("../") {
                    ctx.report(node, "require-relative", "Use `require_relative` instead of `require` with relative path".into());
                }
            }
        }
    })
}

/// Detect `rescue Exception` (too broad, catches everything including SystemExit).
pub fn check_no_rescue_exception(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "rescue" {
            return;
        }
        let Some(exceptions) = node.child_by_field_name("exceptions") else { return };
        let text = node_text(&exceptions, ctx.source);
        if text.contains("Exception") {
            ctx.report(node, "no-rescue-exception", "Avoid `rescue Exception`; use `rescue StandardError` instead".into());
        }
    })
}

/// Detect `sleep` in non-test code.
pub fn check_no_sleep(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    if fp.contains("_test.rb") || fp.contains("_spec.rb") || fp.contains("test_") {
        return vec![];
    }
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" && node.kind() != "identifier" {
            return;
        }
        let text = if node.kind() == "call" {
            node.child_by_field_name("method").map(|m| node_text(&m, ctx.source))
        } else {
            Some(node_text(node, ctx.source))
        };
        if text == Some("sleep") {
            ctx.report(node, "no-sleep", "Avoid `sleep` in production code".into());
        }
    })
}
