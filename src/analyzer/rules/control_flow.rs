use tree_sitter::Tree;

use crate::types::{Issue, Severity};

use super::run_check;

pub fn check_no_unreachable(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "statement_block" {
            let mut found_terminator = false;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let kind = child.kind();
                if kind == "{" || kind == "}" {
                    continue;
                }
                if found_terminator && kind != "comment" {
                    ctx.report(&child, "no-unreachable", "Unreachable code detected".into());
                    found_terminator = false;
                }
                if matches!(kind, "return_statement" | "throw_statement" | "break_statement" | "continue_statement") {
                    found_terminator = true;
                }
            }
        }
    })
}
