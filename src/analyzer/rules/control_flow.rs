use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line};
use crate::types::{Issue, Severity};

/// no-unreachable: Disallow unreachable code after return/throw/break/continue.
pub fn check_no_unreachable(
    tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    fn visit(node: &Node, source: &[u8], file_path: &str, severity: Severity, issues: &mut Vec<Issue>) {
        if node.kind() == "statement_block" {
            let mut found_terminator = false;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let kind = child.kind();
                if kind == "{" || kind == "}" {
                    continue;
                }
                if found_terminator && kind != "comment" {
                    issues.push(Issue {
                        file: file_path.to_string(),
                        line: node_line(&child),
                        column: node_column(&child),
                        severity,
                        rule: "no-unreachable".to_string(),
                        message: "Unreachable code detected".to_string(),
                    });
                    found_terminator = false; // Only report once per block
                }
                if matches!(
                    kind,
                    "return_statement"
                        | "throw_statement"
                        | "break_statement"
                        | "continue_statement"
                ) {
                    found_terminator = true;
                }
            }
        }

        let _ = source;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, file_path, severity, issues);
        }
    }

    visit(&tree.root_node(), source, file_path, severity, &mut issues);
    issues
}
