use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect local variable shadowing an imported package name.
pub fn check_no_shadow_import(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    let root = tree.root_node();
    let mut imports: Vec<String> = Vec::new();
    collect_imports(&root, source, &mut imports);
    if imports.is_empty() {
        return vec![];
    }
    let mut issues = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "short_var_declaration" {
            issues.extend(find_shadowed_imports(&node, source, &imports, fp, sev));
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    issues
}

fn find_shadowed_imports(
    node: &tree_sitter::Node,
    source: &[u8],
    imports: &[String],
    fp: &str,
    sev: Severity,
) -> Vec<Issue> {
    use crate::analyzer::ast_utils::{node_column, node_line};
    let Some(left) = node.child_by_field_name("left") else { return vec![] };
    let mut issues = Vec::new();
    let mut cursor = left.walk();
    for child in left.children(&mut cursor) {
        if child.kind() != "identifier" {
            continue;
        }
        let name = node_text(&child, source);
        if imports.iter().any(|imp| imp == name) {
            issues.push(Issue {
                file: fp.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity: sev,
                rule: "no-shadow-import".to_string(),
                message: format!("Variable `{name}` shadows imported package"),
            });
        }
    }
    issues
}

fn collect_imports(node: &tree_sitter::Node, source: &[u8], imports: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_declaration" | "import_spec_list" => collect_imports(&child, source, imports),
            "import_spec" => collect_import_spec(&child, source, imports),
            _ => {}
        }
    }
}

fn collect_import_spec(child: &tree_sitter::Node, source: &[u8], imports: &mut Vec<String>) {
    if let Some(name) = child.child_by_field_name("name") {
        let alias = node_text(&name, source);
        if alias != "_" && alias != "." {
            imports.push(alias.to_string());
        }
    } else if let Some(path) = child.child_by_field_name("path") {
        let path_text = node_text(&path, source).trim_matches('"');
        if let Some(last) = path_text.rsplit('/').next() {
            imports.push(last.to_string());
        }
    }
}

/// Detect `if a { if b { ... } }` that can be merged with `&&`.
pub fn check_collapsible_if_go(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| c.kind() == "else_clause");
        if has_else {
            return;
        }
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        if consequence.kind() != "block" {
            return;
        }
        let mut block_cursor = consequence.walk();
        let stmts: Vec<_> = consequence.children(&mut block_cursor)
            .filter(|c| !matches!(c.kind(), "{" | "}" | "comment"))
            .collect();
        let inner = if stmts.len() == 1 && stmts[0].kind() == "statement_list" {
            let sl = &stmts[0];
            let mut sl_cursor = sl.walk();
            sl.children(&mut sl_cursor).collect::<Vec<_>>()
        } else {
            stmts
        };
        if inner.len() == 1 && inner[0].kind() == "if_statement" {
            let inner_if = &inner[0];
            let mut inner_cursor = inner_if.walk();
            let inner_has_else = inner_if.children(&mut inner_cursor).any(|c| c.kind() == "else_clause");
            if !inner_has_else {
                ctx.report(node, "collapsible-if", "Nested `if` can be merged with `&&`".into());
            }
        }
    })
}

/// Detect superfluous else after return.
pub fn check_superfluous_else_go(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| c.kind() == "else_clause");
        if !has_else {
            return;
        }
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        if ends_with_return_go(&consequence) {
            ctx.report(node, "superfluous-else", "Remove `else` after `return`; dedent the else body".into());
        }
    })
}

fn ends_with_return_go(block: &tree_sitter::Node) -> bool {
    let mut cursor = block.walk();
    let children: Vec<_> = block.children(&mut cursor).collect();
    for child in children.iter().rev() {
        if child.kind() == "statement_list" {
            let mut sl_cursor = child.walk();
            let stmts: Vec<_> = child.children(&mut sl_cursor).collect();
            if let Some(last) = stmts.last() {
                return matches!(last.kind(), "return_statement" | "break_statement" | "continue_statement");
            }
        }
        if matches!(child.kind(), "return_statement" | "break_statement" | "continue_statement") {
            return true;
        }
    }
    false
}
