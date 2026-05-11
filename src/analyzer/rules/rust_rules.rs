use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

fn is_in_test_module(node: &Node, source: &[u8]) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == "mod_item" {
            if let Some(name) = n.child_by_field_name("name") {
                if node_text(&name, source) == "tests" {
                    return true;
                }
            }
        }
        current = n.parent();
    }
    false
}

pub fn check_no_unsafe(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "unsafe_block" {
            ctx.report(node, "no-unsafe", "Avoid `unsafe` block".into());
        }
    })
}

pub fn check_no_unwrap(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if func.kind() != "field_expression" {
            return;
        }
        let Some(field) = func.child_by_field_name("field") else { return };
        if node_text(&field, ctx.source) == "unwrap" && !is_in_test_module(node, ctx.source) {
            ctx.report(node, "no-unwrap", "Avoid `.unwrap()`, use `?` or handle the error explicitly".into());
        }
    })
}

pub fn check_no_dbg(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "macro_invocation" {
            return;
        }
        let Some(m) = node.child_by_field_name("macro") else { return };
        if node_text(&m, ctx.source) == "dbg" {
            ctx.report(node, "no-dbg", "Unexpected `dbg!()` macro".into());
        }
    })
}

pub fn check_no_todo(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "macro_invocation" {
            return;
        }
        let Some(m) = node.child_by_field_name("macro") else { return };
        let name = node_text(&m, ctx.source);
        if name == "todo" || name == "unimplemented" {
            ctx.report(node, "no-todo", format!("Unexpected `{name}!()` macro"));
        }
    })
}

pub fn check_no_println(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "macro_invocation" {
            return;
        }
        let Some(m) = node.child_by_field_name("macro") else { return };
        let name = node_text(&m, ctx.source);
        if matches!(name, "println" | "print" | "eprintln" | "eprint") {
            ctx.report(node, "no-println", format!("Unexpected `{name}!()` macro, use a logging framework"));
        }
    })
}

/// Detect `if cond { true } else { false }` patterns (needless bool).
/// Inspired by Clippy's needless_bool.
pub fn check_needless_bool(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_expression" {
            return;
        }
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        let Some(alternative) = node.child_by_field_name("alternative") else { return };

        let then_val = extract_single_bool_return(&consequence, ctx.source);
        let else_val = extract_single_bool_return(&alternative, ctx.source);

        match (then_val, else_val) {
            (Some(true), Some(false)) => {
                ctx.report(node, "needless-bool", "This if-else returns bools; use the condition directly".into());
            }
            (Some(false), Some(true)) => {
                ctx.report(node, "needless-bool", "This if-else returns bools; negate the condition".into());
            }
            _ => {}
        }
    })
}

fn extract_single_bool_return(block: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    let mut cursor = block.walk();
    let stmts: Vec<_> = block.children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "{" | "}" | "else"))
        .collect();
    if stmts.len() != 1 {
        return None;
    }
    let stmt = &stmts[0];
    let text = node_text(stmt, source).trim();
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn check_no_empty_function_rust(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_item" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() == "block" && !has_rust_statements(&body) {
            ctx.report(node, "no-empty-function", "Unexpected empty function".into());
        }
    })
}

fn has_rust_statements(block: &tree_sitter::Node) -> bool {
    let mut cursor = block.walk();
    let result = block.children(&mut cursor).any(|c| {
        let k = c.kind();
        k != "{" && k != "}" && k != "line_comment" && k != "block_comment"
    });
    result
}

/// Detect explicit `return x;` as the last statement in a function body.
/// Inspired by Clippy's needless_return.
pub fn check_needless_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_item" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "{" | "}"))
            .collect();
        let Some(last) = stmts.last() else { return };
        // The last statement may be an expression_statement containing a return_expression,
        // or a return_expression directly.
        let target = if last.kind() == "expression_statement" {
            if let Some(child) = last.child(0) { child } else { return }
        } else {
            *last
        };
        if target.kind() == "return_expression" {
            ctx.report(node, "needless-return", "Unnecessary `return` in tail position; remove the `return` keyword".into());
        }
    })
}

/// Detect `x == true`, `x == false`, `x != true`, `x != false`.
/// Inspired by Clippy's bool_comparison.
pub fn check_bool_comparison(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op_node) = node.child_by_field_name("operator") else { return };
        let op = node_text(&op_node, ctx.source);
        if op != "==" && op != "!=" {
            return;
        }
        let Some(left) = node.child_by_field_name("left") else { return };
        let Some(right) = node.child_by_field_name("right") else { return };
        let lt = node_text(&left, ctx.source);
        let rt = node_text(&right, ctx.source);
        if lt == "true" || lt == "false" || rt == "true" || rt == "false" {
            ctx.report(node, "bool-comparison", format!("Redundant comparison with boolean literal; simplify the expression"));
        }
    })
}

