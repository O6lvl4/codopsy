use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

pub fn check_no_any(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "predefined_type" && node_text(node, ctx.source) == "any" {
            ctx.report(node, "no-any", "Unexpected `any` type".into());
        }
    })
}

pub fn check_no_console(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(callee) = node.child_by_field_name("function") else { return };
        if callee.kind() != "member_expression" {
            return;
        }
        let Some(obj) = callee.child_by_field_name("object") else { return };
        if node_text(&obj, ctx.source) == "console" {
            ctx.report(node, "no-console", "Unexpected console statement".into());
        }
    })
}

pub fn check_no_var(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "variable_declaration" {
            return;
        }
        let Some(first) = node.child(0) else { return };
        if node_text(&first, ctx.source) == "var" {
            ctx.report(node, "no-var", "Unexpected var, use let or const instead".into());
        }
    })
}

pub fn check_eqeqeq(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if op_text == "==" || op_text == "!=" {
            let suggested = if op_text == "==" { "===" } else { "!==" };
            ctx.report(&op, "eqeqeq", format!("Expected '{suggested}' instead of '{op_text}'"));
        }
    })
}

pub fn check_no_empty_function(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if !crate::analyzer::ast_utils::is_function_node(node) {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() == "statement_block" && !has_statements(&body) {
            ctx.report(node, "no-empty-function", "Unexpected empty function".into());
        }
    })
}

fn has_statements(block: &Node) -> bool {
    let mut cursor = block.walk();
    let result = block
        .children(&mut cursor)
        .any(|c| c.kind() != "{" && c.kind() != "}" && c.kind() != "comment");
    result
}

pub fn check_no_nested_ternary(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "ternary_expression" && contains_ternary(node) {
            ctx.report(node, "no-nested-ternary", "Do not nest ternary expressions".into());
        }
    })
}

fn contains_ternary(node: &Node) -> bool {
    let mut cursor = node.walk();
    let result = node.children(&mut cursor).any(|c| {
        c.kind() == "ternary_expression" || contains_ternary(&c)
    });
    result
}
