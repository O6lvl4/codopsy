use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

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
        if node_text(&field, ctx.source) == "unwrap" {
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

/// Detect `if a { if b { ... } }` where outer if has only a single inner if.
/// Inspired by Clippy's collapsible_if.
pub fn check_collapsible_if(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_expression" {
            return;
        }
        // Must not have an else branch to be collapsible.
        if node.child_by_field_name("alternative").is_some() {
            return;
        }
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        if consequence.kind() != "block" {
            return;
        }
        let mut cursor = consequence.walk();
        let stmts: Vec<_> = consequence.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "{" | "}"))
            .collect();
        if stmts.len() != 1 {
            return;
        }
        let inner = &stmts[0];
        // The inner node may be an expression_statement wrapping an if_expression.
        let target = if inner.kind() == "expression_statement" {
            if let Some(child) = inner.child(0) { child } else { return }
        } else {
            *inner
        };
        if target.kind() == "if_expression" && target.child_by_field_name("alternative").is_none() {
            ctx.report(node, "collapsible-if", "These `if` statements can be collapsed into `if a && b { ... }`".into());
        }
    })
}

/// Detect `match x { Pattern => ..., _ => () }` with exactly 2 arms where second is wildcard returning unit.
/// Inspired by Clippy's single_match. Suggest using `if let`.
pub fn check_single_match(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "match_expression" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let arms: Vec<_> = body.children(&mut cursor)
            .filter(|c| c.kind() == "match_arm")
            .collect();
        if arms.len() != 2 {
            return;
        }
        let last_arm = &arms[1];
        // Check if the last arm's pattern is a wildcard.
        let Some(pattern) = last_arm.child_by_field_name("pattern") else { return };
        if pattern.kind() != "_" && node_text(&pattern, ctx.source).trim() != "_" {
            return;
        }
        // Check if the last arm's value is unit `()`.
        let Some(value) = last_arm.child_by_field_name("value") else { return };
        let val_text = node_text(&value, ctx.source).trim();
        if val_text == "()" || val_text.is_empty() {
            ctx.report(node, "single-match", "This `match` has a single non-wildcard arm; use `if let` instead".into());
        }
    })
}

/// Detect `match opt { Some(x) => Some(f(x)), None => None }` pattern.
/// Inspired by Clippy's manual_map. Suggest using `.map()`.
pub fn check_manual_map(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "match_expression" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let arms: Vec<_> = body.children(&mut cursor)
            .filter(|c| c.kind() == "match_arm")
            .collect();
        if arms.len() != 2 {
            return;
        }
        let arm0_text = arm_pattern_text(&arms[0], ctx.source);
        let arm1_text = arm_pattern_text(&arms[1], ctx.source);
        let arm0_val = arm_value_text(&arms[0], ctx.source);
        let arm1_val = arm_value_text(&arms[1], ctx.source);

        let is_some_none = (arm0_text.starts_with("Some") && arm1_text == "None"
            && arm0_val.starts_with("Some") && arm1_val == "None")
            || (arm0_text == "None" && arm1_text.starts_with("Some")
                && arm0_val == "None" && arm1_val.starts_with("Some"));

        if is_some_none {
            ctx.report(node, "manual-map", "This `match` manually maps `Some`/`None`; use `.map()` instead".into());
        }
    })
}

fn arm_pattern_text<'a>(arm: &tree_sitter::Node, source: &'a [u8]) -> &'a str {
    arm.child_by_field_name("pattern")
        .map(|p| node_text(&p, source).trim())
        .unwrap_or("")
}

fn arm_value_text<'a>(arm: &tree_sitter::Node, source: &'a [u8]) -> &'a str {
    arm.child_by_field_name("value")
        .map(|v| node_text(&v, source).trim())
        .unwrap_or("")
}

/// Detect `.clone().clone()` double-clone pattern.
/// A simplified version of Clippy's redundant_clone (full scope analysis not feasible via AST alone).
pub fn check_redundant_clone(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if func.kind() != "field_expression" {
            return;
        }
        let Some(field) = func.child_by_field_name("field") else { return };
        if node_text(&field, ctx.source) != "clone" {
            return;
        }
        // Check if the receiver is also a .clone() call.
        let Some(receiver) = func.child_by_field_name("value") else { return };
        if receiver.kind() != "call_expression" {
            return;
        }
        let Some(inner_func) = receiver.child_by_field_name("function") else { return };
        if inner_func.kind() != "field_expression" {
            return;
        }
        let Some(inner_field) = inner_func.child_by_field_name("field") else { return };
        if node_text(&inner_field, ctx.source) == "clone" {
            ctx.report(node, "redundant-clone", "Redundant `.clone().clone()`; a single `.clone()` suffices".into());
        }
    })
}

/// Detect self-comparison like `x == x`, `x != x`, `x >= x`, etc.
/// Inspired by Clippy's eq_op.
pub fn check_eq_op(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op_node) = node.child_by_field_name("operator") else { return };
        let op = node_text(&op_node, ctx.source);
        if !matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=" | "&" | "|" | "^") {
            return;
        }
        let Some(left) = node.child_by_field_name("left") else { return };
        let Some(right) = node.child_by_field_name("right") else { return };
        let lt = node_text(&left, ctx.source).trim();
        let rt = node_text(&right, ctx.source).trim();
        if !lt.is_empty() && lt == rt {
            ctx.report(node, "eq-op", format!("Both sides of `{op}` are identical: `{lt}`"));
        }
    })
}
