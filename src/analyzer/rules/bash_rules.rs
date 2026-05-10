use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect unquoted variable expansions: `$var` instead of `"$var"`.
/// Inspired by ShellCheck SC2086.
pub fn check_unquoted_expansion(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        // simple_expansion = $var (without quotes)
        if node.kind() != "simple_expansion" {
            return;
        }
        // If parent is a string (double-quoted), it's fine
        if let Some(parent) = node.parent() {
            if parent.kind() == "string" || parent.kind() == "string_expansion" {
                return;
            }
            // Inside [[ ]] is also fine
            if parent.kind() == "test_command" {
                return;
            }
        }
        let var = node_text(node, ctx.source);
        ctx.report(
            node,
            "unquoted-expansion",
            format!("Unquoted variable {var}; use \"{var}\" to prevent word splitting"),
        );
    })
}

/// Detect `eval` usage.
/// Inspired by ShellCheck SC2091.
pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "command" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) == "eval" {
            ctx.report(node, "no-eval", "Avoid `eval`; it's a security risk and makes code harder to debug".into());
        }
    })
}

/// Detect `cd` without `||` error handling.
/// Inspired by ShellCheck SC2164.
pub fn check_cd_without_or(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "command" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) != "cd" {
            return;
        }
        // Check if parent is a `list` with `||` (error handling)
        if let Some(parent) = node.parent() {
            if parent.kind() == "list" {
                // list with || is fine
                let mut cursor = parent.walk();
                let has_or = parent.children(&mut cursor).any(|c| node_text(&c, ctx.source) == "||");
                if has_or {
                    return;
                }
            }
        }
        ctx.report(
            node,
            "cd-without-or",
            "Use `cd dir || exit 1` to handle failure".into(),
        );
    })
}

/// Detect useless `cat`: `cat file | cmd` instead of `cmd < file`.
/// Inspired by ShellCheck SC2002.
pub fn check_useless_cat(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "pipeline" {
            return;
        }
        // First command in pipeline
        let Some(first) = node.child(0) else { return };
        if first.kind() != "command" {
            return;
        }
        let Some(name) = first.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) == "cat" {
            // cat with exactly one argument piped into something
            let mut cursor = first.walk();
            let args: Vec<_> = first.children(&mut cursor)
                .filter(|c| c.kind() == "word" && c.id() != name.id())
                .collect();
            if args.len() == 1 {
                ctx.report(
                    &first,
                    "useless-cat",
                    "Useless `cat`; use `cmd < file` instead of `cat file | cmd`".into(),
                );
            }
        }
    })
}

/// Detect `rm -rf` with variable expansion (dangerous).
pub fn check_dangerous_rm(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "command" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) != "rm" {
            return;
        }
        let full = node_text(node, ctx.source);
        let is_recursive_force = full.contains("-rf") || full.contains("-r -f") || full.contains("-fr");
        let has_expansion = full.contains('$') || full.contains('*');
        if is_recursive_force && has_expansion {
            ctx.report(
                node,
                "dangerous-rm",
                "Dangerous `rm -rf` with variable/glob expansion; validate paths first".into(),
            );
        }
    })
}

/// Detect `[ x = y ]` instead of `[ x = "y" ]` or `[[ x == y ]]`.
/// Inspired by ShellCheck SC2039/SC3010.
pub fn check_test_equals(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "test_command" {
            return;
        }
        let text = node_text(node, ctx.source);
        // [ ... == ... ] is bashism, should use = in [ ] or use [[ ]]
        if text.starts_with("[ ") && text.contains(" == ") {
            ctx.report(
                node,
                "test-equals",
                "Use `=` instead of `==` in `[ ]`, or use `[[ ]]` for bash-specific features".into(),
            );
        }
    })
}

/// Detect missing `set -e` / `set -euo pipefail` at the top of scripts.
pub fn check_no_set_e(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    let root = tree.root_node();
    let src_text = std::str::from_utf8(source).unwrap_or("");

    // Check if any `set -e` or `set -euo pipefail` appears in the first 10 lines
    let has_set_e = src_text.lines().take(10).any(|line| {
        let t = line.trim();
        t.starts_with("set -e") || t.starts_with("set -o errexit")
    });

    if !has_set_e && root.child_count() > 0 {
        vec![Issue {
            file: fp.to_string(),
            line: 1,
            column: 1,
            severity: sev,
            rule: "no-set-e".to_string(),
            message: "Consider adding `set -euo pipefail` for safer error handling".to_string(),
        }]
    } else {
        vec![]
    }
}
