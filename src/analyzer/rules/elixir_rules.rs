use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `IO.inspect()` calls (debug-only function).
pub fn check_no_io_inspect(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(target) = node.child_by_field_name("target") else { return };
        if target.kind() != "dot" {
            return;
        }
        let Some(left) = target.child_by_field_name("left") else { return };
        let Some(right) = target.child_by_field_name("right") else { return };
        if left.kind() == "alias" && node_text(&left, ctx.source) == "IO"
            && right.kind() == "identifier" && node_text(&right, ctx.source) == "inspect"
        {
            ctx.report(node, "no-io-inspect", "Avoid `IO.inspect()`, use `Logger` instead".into());
        }
    })
}

/// Detect `IO.puts()` calls (debug-only function).
pub fn check_no_io_puts(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(target) = node.child_by_field_name("target") else { return };
        if target.kind() != "dot" {
            return;
        }
        let Some(left) = target.child_by_field_name("left") else { return };
        let Some(right) = target.child_by_field_name("right") else { return };
        if left.kind() == "alias" && node_text(&left, ctx.source) == "IO"
            && right.kind() == "identifier" && node_text(&right, ctx.source) == "puts"
        {
            ctx.report(node, "no-io-puts", "Avoid `IO.puts()`, use `Logger` instead".into());
        }
    })
}

/// Detect `raise` inside `with` blocks.
pub fn check_no_raise_in_with(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(target) = node.child_by_field_name("target") else { return };
        if target.kind() != "identifier" || node_text(&target, ctx.source) != "raise" {
            return;
        }
        // Walk up to check if inside a `with` block
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "call" {
                if let Some(t) = parent.child_by_field_name("target") {
                    if t.kind() == "identifier" && node_text(&t, ctx.source) == "with" {
                        ctx.report(
                            node,
                            "no-raise-in-with",
                            "Avoid `raise` inside `with`; use pattern matching to handle errors".into(),
                        );
                        return;
                    }
                }
            }
            current = parent.parent();
        }
    })
}

/// Detect piping into anonymous functions (`|> fn ... end` or `|> &(...)`).
pub fn check_pipe_into_anonymous(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_operator" {
            return;
        }
        let Some(operator) = node.child_by_field_name("operator") else { return };
        if node_text(&operator, ctx.source) != "|>" {
            return;
        }
        let Some(right) = node.child_by_field_name("right") else { return };
        if matches!(right.kind(), "anonymous_function" | "capture") {
            ctx.report(
                node,
                "pipe-into-anonymous",
                "Avoid piping into anonymous functions; extract to a named function".into(),
            );
        }
    })
}
