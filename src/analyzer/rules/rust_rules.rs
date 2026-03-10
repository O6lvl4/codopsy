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
