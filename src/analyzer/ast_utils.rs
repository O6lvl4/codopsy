use tree_sitter::{Node, Parser, Tree};

pub use super::language::{SourceLanguage, get_language};

pub fn parse_source(source: &str, language: SourceLanguage) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_lang = language.tree_sitter_language();
    if parser.set_language(&ts_lang).is_err() {
        eprintln!("Failed to set language for {:?}", language);
        return None;
    }
    parser.parse(source.as_bytes(), None)
}

/// Check if a node kind is a function-like construct.
fn is_function_kind(kind: &str) -> bool {
    matches!(
        kind,
        // JS/TS (also shared by Go, PHP, Scala, Haskell)
        "function_declaration"
            | "function"
            | "function_expression"
            | "arrow_function"
            | "method_definition"
            | "generator_function_declaration"
            | "generator_function"
            // Rust
            | "function_item"
            | "closure_expression"
            // Go
            | "method_declaration"
            | "func_literal"
            // Python / C / C++ / PHP / Scala
            | "function_definition"
            // Java / C#
            | "constructor_declaration"
            | "lambda_expression"
            // Ruby
            | "method"
            | "singleton_method"
            | "lambda"
            // PHP
            | "anonymous_function_creation_expression"
            // OCaml
            | "let_binding"
            | "value_definition"
            // Almide
            | "test_declaration"
            // Erlang
            | "fun_expr"
            // Gleam: uses "function" which is already listed above
    )
}

/// Check if a node is a function definition.
/// Skips redundant nested function nodes (e.g. JS `function` inside `function_declaration`,
/// Lua `function` inside `function_declaration`) to avoid double-counting.
pub fn is_function_node(node: &Node) -> bool {
    let kind = node.kind();
    if !is_function_kind(kind) {
        return false;
    }
    // `function` / `generator_function` nodes that are the direct body of a
    // `function_declaration` / `generator_function_declaration` are redundant.
    if matches!(kind, "function" | "generator_function") {
        if let Some(parent) = node.parent() {
            if is_function_kind(parent.kind()) {
                return false;
            }
        }
    }
    true
}

/// Check if a node is a Clojure function definition (defn, defn-).
pub fn is_clojure_defn(node: &Node, source: &[u8]) -> bool {
    if node.kind() != "list_lit" {
        return false;
    }
    // First sym_lit child with name "defn" or "defn-"
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "sym_lit" {
            let text = node_text(&child, source);
            return matches!(text, "defn" | "defn-" | "defmacro");
        }
    }
    false
}

/// Get the function name from a Clojure defn form.
pub fn clojure_defn_name(node: &Node, source: &[u8]) -> String {
    let mut cursor = node.walk();
    let mut found_defn = false;
    for child in node.children(&mut cursor) {
        if child.kind() == "sym_lit" {
            if !found_defn {
                found_defn = true;
                continue;
            }
            return node_text(&child, source).to_string();
        }
    }
    "(anonymous)".to_string()
}

/// Check if a node is an Erlang function declaration.
pub fn is_erlang_fun_decl(node: &Node) -> bool {
    node.kind() == "fun_decl"
}

/// Get the function name from an Erlang fun_decl node.
pub fn erlang_fun_name(node: &Node, source: &[u8]) -> String {
    // fun_decl > function_clause > name: atom
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_clause" {
            if let Some(name) = child.child_by_field_name("name") {
                return node_text(&name, source).to_string();
            }
        }
    }
    "(anonymous)".to_string()
}

/// Check if a tree-sitter `call` node represents an Elixir function definition.
/// Elixir uses `def`, `defp`, `defmacro`, `defmacrop` as function definition calls.
pub fn is_elixir_def_call(node: &Node, source: &[u8]) -> bool {
    if node.kind() != "call" {
        return false;
    }
    if let Some(target) = node.child_by_field_name("target") {
        let text = node_text(&target, source);
        return matches!(text, "def" | "defp" | "defmacro" | "defmacrop");
    }
    false
}

/// Get the function name from an Elixir def call node.
pub fn elixir_def_name(node: &Node, source: &[u8]) -> String {
    // arguments > identifier (simple case)
    // arguments > call > target > identifier (case with params)
    if let Some(args) = find_child_by_kind(node, "arguments") {
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            match child.kind() {
                "identifier" => return node_text(&child, source).to_string(),
                "call" => {
                    if let Some(target) = child.child_by_field_name("target") {
                        return node_text(&target, source).to_string();
                    }
                }
                _ => {}
            }
        }
    }
    "(anonymous)".to_string()
}

/// Get a human-readable function name from a function node.
pub fn get_function_name<'a>(node: &Node<'a>, source: &'a [u8]) -> String {
    let kind = node.kind();

    match kind {
        "function_item" | "function_declaration" | "generator_function_declaration"
        | "function_definition" | "method_declaration" | "constructor_declaration" => {
            name_field_or(node, source, "(anonymous)")
        }
        "test_declaration" => name_from_child_kind(node, source, "string_literal", "(test)"),
        "closure_expression" => name_from_closure(node, source),
        "method_definition" | "method" | "singleton_method" => name_from_method(node, source),
        "arrow_function" | "function_expression" | "function" | "generator_function"
        | "lambda_expression" | "anonymous_function_creation_expression" | "lambda"
        | "func_literal" => {
            name_from_expr(node, source)
        }
        "let_binding" | "value_definition" => name_field_or(node, source, "(anonymous)"),
        _ => "(anonymous)".to_string(),
    }
}

fn name_field_or(node: &Node, source: &[u8], fallback: &str) -> String {
    // Direct "name" field (most languages)
    if let Some(n) = node.child_by_field_name("name") {
        return node_text(&n, source).to_string();
    }
    // PHP uses "function_name" child
    if let Some(n) = find_child_by_kind(node, "function_name") {
        return node_text(&n, source).to_string();
    }
    // C/C++: name is inside "declarator" field (possibly nested as function_declarator)
    if let Some(declarator) = node.child_by_field_name("declarator") {
        return extract_declarator_name(&declarator, source)
            .unwrap_or_else(|| fallback.to_string());
    }
    fallback.to_string()
}

/// Extract the function name from a C/C++ declarator node.
/// Handles `function_declarator(declarator: identifier)` nesting.
fn extract_declarator_name(node: &Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => Some(node_text(node, source).to_string()),
        "function_declarator" | "pointer_declarator" | "reference_declarator" => {
            node.child_by_field_name("declarator")
                .and_then(|d| extract_declarator_name(&d, source))
        }
        _ => {
            // Try "declarator" field generically
            node.child_by_field_name("declarator")
                .and_then(|d| extract_declarator_name(&d, source))
                .or_else(|| {
                    // Fallback: first identifier child
                    find_child_by_kind(node, "identifier")
                        .map(|n| node_text(&n, source).to_string())
                })
        }
    }
}

fn find_child_by_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    (0..node.child_count()).find_map(|i| {
        node.child(i).filter(|c| c.kind() == kind)
    })
}

fn name_from_child_kind(node: &Node, source: &[u8], kind: &str, fallback: &str) -> String {
    find_child_by_kind(node, kind)
        .map(|n| node_text(&n, source).to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn name_from_closure(node: &Node, source: &[u8]) -> String {
    node.parent()
        .filter(|p| p.kind() == "let_declaration")
        .and_then(|p| p.child_by_field_name("pattern"))
        .map(|pat| node_text(&pat, source).to_string())
        .unwrap_or_else(|| "(closure)".to_string())
}

fn name_from_method(node: &Node, source: &[u8]) -> String {
    let Some(name_node) = node.child_by_field_name("name") else {
        return "(anonymous)".to_string();
    };
    let text = node_text(&name_node, source);
    if let Some(first_child) = node.child(0) {
        let prefix = node_text(&first_child, source);
        if prefix == "get" || prefix == "set" {
            return format!("{prefix} {text}");
        }
    }
    text.to_string()
}

fn name_from_expr(node: &Node, source: &[u8]) -> String {
    let kind = node.kind();
    if kind == "function_expression" || kind == "function" {
        if let Some(name_node) = node.child_by_field_name("name") {
            return node_text(&name_node, source).to_string();
        }
    }
    if let Some(parent) = node.parent() {
        let field = match parent.kind() {
            "variable_declarator" | "short_var_declaration" | "assignment" => "name",
            "pair" => "key",
            _ => return "(anonymous)".to_string(),
        };
        if let Some(n) = parent.child_by_field_name(field) {
            return node_text(&n, source).to_string();
        }
    }
    "(anonymous)".to_string()
}

pub fn node_text<'a>(node: &Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

/// Get line number (1-based) from a tree-sitter node.
pub fn node_line(node: &Node) -> usize {
    node.start_position().row + 1
}

/// Get column number (1-based) from a tree-sitter node.
pub fn node_column(node: &Node) -> usize {
    node.start_position().column + 1
}
