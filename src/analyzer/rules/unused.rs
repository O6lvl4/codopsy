//! Unused import detection.
//!
//! Uses two-pass analysis: collect definitions, then scan for references.

use std::collections::HashSet;
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

fn collect_python_imports(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    match node.kind() {
        "import_statement" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "dotted_name" => {
                        let text = node_text(&child, source);
                        if let Some(last) = text.rsplit('.').next() {
                            imports.push(ImportInfo {
                                name: last.to_string(),
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            });
                        }
                    }
                    "aliased_import" => {
                        if let Some(alias) = child.child_by_field_name("alias") {
                            imports.push(ImportInfo {
                                name: node_text(&alias, source).to_string(),
                                line: alias.start_position().row + 1,
                                column: alias.start_position().column + 1,
                            });
                        } else if let Some(name) = child.child_by_field_name("name") {
                            let text = node_text(&name, source);
                            if let Some(last) = text.rsplit('.').next() {
                                imports.push(ImportInfo {
                                    name: last.to_string(),
                                    line: name.start_position().row + 1,
                                    column: name.start_position().column + 1,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "import_from_statement" => {
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
                    "aliased_import" => {
                        if let Some(alias) = child.child_by_field_name("alias") {
                            imports.push(ImportInfo {
                                name: node_text(&alias, source).to_string(),
                                line: alias.start_position().row + 1,
                                column: alias.start_position().column + 1,
                            });
                        } else if let Some(name) = child.child_by_field_name("name") {
                            imports.push(ImportInfo {
                                name: node_text(&name, source).to_string(),
                                line: name.start_position().row + 1,
                                column: name.start_position().column + 1,
                            });
                        }
                    }
                    "dotted_name" | "identifier" => {
                        let text = node_text(&child, source);
                        if !text.is_empty() && text != "," {
                            imports.push(ImportInfo {
                                name: text.to_string(),
                                line: child.start_position().row + 1,
                                column: child.start_position().column + 1,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
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

fn collect_js_import_clause(node: &Node, source: &[u8], imports: &mut Vec<ImportInfo>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                imports.push(ImportInfo {
                    name: node_text(&child, source).to_string(),
                    line: child.start_position().row + 1,
                    column: child.start_position().column + 1,
                });
            }
            "named_imports" => {
                let mut inner = child.walk();
                for spec in child.children(&mut inner) {
                    if spec.kind() == "import_specifier" {
                        let name_node = spec.child_by_field_name("alias")
                            .or_else(|| spec.child_by_field_name("name"));
                        if let Some(n) = name_node {
                            imports.push(ImportInfo {
                                name: node_text(&n, source).to_string(),
                                line: n.start_position().row + 1,
                                column: n.start_position().column + 1,
                            });
                        }
                    }
                }
            }
            "namespace_import" => {
                if let Some(alias) = child.child(2) {
                    imports.push(ImportInfo {
                        name: node_text(&alias, source).to_string(),
                        line: alias.start_position().row + 1,
                        column: alias.start_position().column + 1,
                    });
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
