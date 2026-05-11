use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `if cond: return True; else: return False` pattern.
pub fn check_simplify_boolean_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        let Some(alternative) = node.child_by_field_name("alternative") else { return };
        if alternative.kind() != "else_clause" {
            return;
        }
        let then_return = sole_return_bool(&consequence, ctx.source);
        let else_return = sole_return_bool_from_else(&alternative, ctx.source);
        if let (Some(then_val), Some(else_val)) = (then_return, else_return) {
            if (then_val && !else_val) || (!then_val && else_val) {
                ctx.report(
                    node,
                    "simplify-boolean-return",
                    "Simplify `if cond: return True; else: return False` to `return cond`".into(),
                );
            }
        }
    })
}

fn sole_return_bool(block: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    if block.kind() != "block" {
        return None;
    }
    let mut cursor = block.walk();
    let stmts: Vec<_> = block.children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
        .collect();
    if stmts.len() != 1 || stmts[0].kind() != "return_statement" {
        return None;
    }
    return_is_bool(&stmts[0], source)
}

fn return_is_bool(ret: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    let mut cursor = ret.walk();
    let values: Vec<_> = ret.children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "return" | "comment"))
        .collect();
    if values.len() != 1 {
        return None;
    }
    match node_text(&values[0], source) {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

fn sole_return_bool_from_else(else_clause: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    let mut cursor = else_clause.walk();
    for child in else_clause.children(&mut cursor) {
        if child.kind() == "block" {
            return sole_return_bool(&child, source);
        }
    }
    None
}

/// Detect `if a: if b:` that can be merged into `if a and b:`.
pub fn check_collapsible_if_python(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| {
            matches!(c.kind(), "else_clause" | "elif_clause")
        });
        if has_else {
            return;
        }
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut body_cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut body_cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if stmts.len() != 1 || stmts[0].kind() != "if_statement" {
            return;
        }
        let inner = &stmts[0];
        let mut inner_cursor = inner.walk();
        let inner_has_else = inner.children(&mut inner_cursor).any(|c| {
            matches!(c.kind(), "else_clause" | "elif_clause")
        });
        if !inner_has_else {
            ctx.report(node, "collapsible-if", "Nested `if` can be merged: `if a and b:`".into());
        }
    })
}

/// Detect superfluous else after return/raise.
pub fn check_superfluous_else(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| c.kind() == "else_clause");
        if !has_else {
            return;
        }
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut body_cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut body_cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if let Some(last) = stmts.last() {
            if matches!(last.kind(), "return_statement" | "raise_statement") {
                ctx.report(node, "superfluous-else", "Remove `else` after `return`/`raise`; dedent the else body".into());
            }
        }
    })
}
