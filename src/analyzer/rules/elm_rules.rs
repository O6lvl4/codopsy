use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `Debug.log` calls left in code.
/// Standard Elm lint: Debug module should not be used in production.
pub fn check_no_debug(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let text = node_text(node, ctx.source);
        if matches!(text, "Debug.log" | "Debug.todo" | "Debug.toString") {
            ctx.report(node, "no-debug", format!("Remove `{text}` before shipping"));
        }
    })
}

/// Detect `Debug.todo` (unfinished implementation).
pub fn check_no_todo(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node_text(node, ctx.source) == "Debug.todo" {
            ctx.report(node, "no-todo", "`Debug.todo` will crash at runtime; implement the function".into());
        }
    })
}

/// Detect unused imports (heuristic: imported module name not referenced).
pub fn check_unused_import(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    // Elm imports are `import Module.Name` or `import Module.Name exposing (..)`
    // Heuristic: if module name doesn't appear anywhere else in the file, it's unused
    let root = tree.root_node();
    let mut issues = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() != "import_clause" {
            continue;
        }
        let text = node_text(&child, source);
        // Extract module name from "import Foo.Bar exposing (..)"
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let module_name = parts[1];
        // Get the last segment for qualified usage check
        let last_segment = module_name.rsplit('.').next().unwrap_or(module_name);

        // Check if this module name appears elsewhere in the source
        let src_text = std::str::from_utf8(source).unwrap_or("");
        let import_line = &text;
        let rest = src_text.replace(import_line, "");
        if !rest.contains(last_segment) {
            issues.push(Issue {
                file: fp.to_string(),
                line: child.start_position().row + 1,
                column: child.start_position().column + 1,
                severity: sev,
                rule: "unused-import".to_string(),
                message: format!("`{module_name}` imported but unused"),
            });
        }
    }
    issues
}
