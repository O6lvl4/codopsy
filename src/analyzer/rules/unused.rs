//! Unused import detection.
//!
//! Uses two-pass analysis: collect definitions, then scan for references.

use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

struct ImportInfo {
    name: String,
    line: usize,
    column: usize,
}

// ─── Python unused imports ──────────────────────────────────────────────

pub fn check_unused_import_python(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    let root = tree.root_node();
    let mut imports: Vec<ImportInfo> = Vec::new();

    collect_python_imports(&root, source, &mut imports);

    let mut issues = Vec::new();
    for imp in &imports {
        if imp.name.starts_with('_') {
            continue;
        }
        if !is_name_used_outside_imports(&root, source, &imp.name) {
            issues.push(Issue {
                file: fp.to_string(),
                line: imp.line,
                column: imp.column,
                severity: sev,
                rule: "unused-import".to_string(),
                message: format!("`{}` imported but unused", imp.name),
            });
        }
    }
    issues
}

fn import_info(node: &Node, source: &[u8]) -> ImportInfo {
    ImportInfo {
        name: node_text(node, source).to_string(),
        line: node.start_position().row + 1,
        column: node.start_position().column + 1,
    }
}

fn import_info_last_segment(node: &Node, source: &[u8]) -> Option<ImportInfo> {
    let text = node_text(node, source);
    text.rsplit('.').next().map(|last| ImportInfo {
        name: last.to_string(),
        line: node.start_position().row + 1,
        column: node.start_position().column + 1,
    })
}

fn collect_aliased_import(child: &Node, source: &[u8], imports: &mut Vec<ImportInfo>, dotted_to_last: bool) {
    if let Some(alias) = child.child_by_field_name("alias") {
        imports.push(import_info(&alias, source));
    } else if let Some(name) = child.child_by_field_name("name") {
        if dotted_to_last {
            if let Some(info) = import_info_last_segment(&name, source) {
                imports.push(info);
            }
        } else {
            imports.push(import_info(&name, source));
        }
    }
}

fn collect_import_statement(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" => {
                if let Some(info) = import_info_last_segment(&child, source) {
                    imports.push(info);
                }
            }
            "aliased_import" => collect_aliased_import(&child, source, imports, true),
            _ => {}
        }
    }
}

fn collect_import_from_statement(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    let mut cursor = node.walk();
    let mut past_import = false;
    for child in node.children(&mut cursor) {
        if node_text(&child, source) == "import" {
            past_import = true;
            continue;
        }
        if !past_import {
            continue;
        }
        match child.kind() {
            "aliased_import" => collect_aliased_import(&child, source, imports, false),
            "dotted_name" | "identifier" => {
                let text = node_text(&child, source);
                if !text.is_empty() && text != "," {
                    imports.push(import_info(&child, source));
                }
            }
            _ => {}
        }
    }
}

fn collect_python_imports(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    match node.kind() {
        "import_statement" => collect_import_statement(node, source, imports),
        "import_from_statement" => collect_import_from_statement(node, source, imports),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_python_imports(&child, source, imports);
    }
}

fn is_name_used_outside_imports(root: &Node, source: &[u8], name: &str) -> bool {
    let mut found = false;
    walk_for_usage(root, source, name, &mut found);
    found
}

fn walk_for_usage(node: &Node, source: &[u8], name: &str, found: &mut bool) {
    if *found {
        return;
    }
    if matches!(node.kind(), "import_statement" | "import_from_statement") {
        return;
    }
    if node.kind() == "identifier" && node_text(node, source) == name {
        *found = true;
        return;
    }
    if node.kind() == "attribute" {
        if let Some(obj) = node.child_by_field_name("object") {
            if node_text(&obj, source) == name {
                *found = true;
                return;
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_usage(&child, source, name, found);
    }
}

// ─── JS/TS unused imports ───────────────────────────────────────────────

pub fn check_unused_import_js(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    let root = tree.root_node();
    let mut imports: Vec<ImportInfo> = Vec::new();

    collect_js_imports(&root, source, &mut imports);

    let mut issues = Vec::new();
    for imp in &imports {
        if imp.name.starts_with('_') {
            continue;
        }
        if !is_js_name_used_outside_imports(&root, source, &imp.name) {
            issues.push(Issue {
                file: fp.to_string(),
                line: imp.line,
                column: imp.column,
                severity: sev,
                rule: "unused-import".to_string(),
                message: format!("`{}` imported but unused", imp.name),
            });
        }
    }
    issues
}

fn collect_js_imports(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    if node.kind() == "import_statement" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_clause" {
                collect_js_import_clause(&child, source, imports);
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_js_imports(&child, source, imports);
    }
}

fn collect_named_imports(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    let mut inner = node.walk();
    for spec in node.children(&mut inner) {
        if spec.kind() != "import_specifier" {
            continue;
        }
        let name_node = spec.child_by_field_name("alias")
            .or_else(|| spec.child_by_field_name("name"));
        if let Some(n) = name_node {
            imports.push(import_info(&n, source));
        }
    }
}

fn collect_js_import_clause(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => imports.push(import_info(&child, source)),
            "named_imports" => collect_named_imports(&child, source, imports),
            "namespace_import" => {
                if let Some(alias) = child.child(2) {
                    imports.push(import_info(&alias, source));
                }
            }
            _ => {}
        }
    }
}

fn is_js_name_used_outside_imports(root: &Node, source: &[u8], name: &str) -> bool {
    let mut found = false;
    walk_js_usage(root, source, name, &mut found);
    found
}

fn walk_js_usage(node: &Node, source: &[u8], name: &str, found: &mut bool) {
    if *found {
        return;
    }
    if matches!(node.kind(), "import_statement" | "import_declaration") {
        return;
    }
    if node.kind() == "identifier" && node_text(node, source) == name {
        *found = true;
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_js_usage(&child, source, name, found);
    }
}
