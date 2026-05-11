use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `if a { if b { ... } }` where outer if has only a single inner if.
pub fn check_collapsible_if(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_expression" {
            return;
        }
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
        let target = if inner.kind() == "expression_statement" {
            if let Some(child) = inner.child(0) { child } else { return }
        } else {
            *inner
        };
        if target.kind() == "if_expression" && target.child_by_field_name("alternative").is_none() {
            let outer_is_if_let = node.child_by_field_name("condition")
                .map_or(false, |c| c.kind() == "let_condition" || c.kind() == "let_chain");
            let inner_is_if_let = target.child_by_field_name("condition")
                .map_or(false, |c| c.kind() == "let_condition" || c.kind() == "let_chain");
            if outer_is_if_let || inner_is_if_let {
                return;
            }
            ctx.report(node, "collapsible-if", "These `if` statements can be collapsed into `if a && b { ... }`".into());
        }
    })
}

/// Detect `match x { Pattern => ..., _ => () }` with exactly 2 arms where second is wildcard.
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
        let Some(pattern) = last_arm.child_by_field_name("pattern") else { return };
        if pattern.kind() != "_" && node_text(&pattern, ctx.source).trim() != "_" {
            return;
        }
        let Some(value) = last_arm.child_by_field_name("value") else { return };
        let val_text = node_text(&value, ctx.source).trim();
        if val_text == "()" || val_text.is_empty() {
            ctx.report(node, "single-match", "This `match` has a single non-wildcard arm; use `if let` instead".into());
        }
    })
}

/// Detect `match opt { Some(x) => Some(f(x)), None => None }` pattern.
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
