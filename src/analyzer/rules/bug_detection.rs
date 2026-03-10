use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

pub fn check_no_debugger(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "debugger_statement" {
            ctx.report(node, "no-debugger", "Unexpected 'debugger' statement".into());
        }
    })
}

pub fn check_no_duplicate_case(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "switch_body" {
            let mut seen = std::collections::HashSet::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "switch_case" {
                    if let Some(value) = child.child_by_field_name("value") {
                        let text = node_text(&value, ctx.source).to_string();
                        if !seen.insert(text.clone()) {
                            ctx.report(&child, "no-duplicate-case", format!("Duplicate case label: {text}"));
                        }
                    }
                }
            }
        }
    })
}

pub fn check_no_self_assign(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "assignment_expression" {
            if let Some(op) = node.child_by_field_name("operator") {
                if node_text(&op, ctx.source) == "=" {
                    if let (Some(left), Some(right)) = (
                        node.child_by_field_name("left"),
                        node.child_by_field_name("right"),
                    ) {
                        let lt = node_text(&left, ctx.source);
                        let rt = node_text(&right, ctx.source);
                        if lt == rt {
                            ctx.report(node, "no-self-assign", format!("'{lt}' is assigned to itself"));
                        }
                    }
                }
            }
        }
    })
}

pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "call_expression" {
            if let Some(callee) = node.child_by_field_name("function") {
                if callee.kind() == "identifier" && node_text(&callee, ctx.source) == "eval" {
                    ctx.report(node, "no-eval", "eval() is not allowed".into());
                }
            }
        }
    })
}
