//! Per-language declaration recognisers.
//!
//! Some grammars do not model a declaration as its own node kind: an Elixir
//! `def` is a plain call, a Clojure `defn` is a list literal. Others reuse a
//! node kind that means something different elsewhere (`instance` is both a
//! Haskell instance head and a Lean 4 one), so the caller must already know the
//! language. Either way these cannot go in the shared function-kind table.

use tree_sitter::Node;

use super::ast_utils::{find_child_by_kind, node_text};

/// Check if a node is a Lean 4 `instance` declaration.
///
/// tree-sitter-haskell also has an `instance` node kind, so this must only be
/// called once the source language is known to be Lean.
pub fn is_lean_instance(node: &Node) -> bool {
    node.kind() == "instance" && node.is_named()
}

/// Get the name of a Lean 4 `instance`. The name is optional in Lean, in which
/// case the instance is identified by the type it implements.
pub fn lean_instance_name(node: &Node, source: &[u8]) -> String {
    if let Some(name) = node.child_by_field_name("name") {
        return node_text(&name, source).to_string();
    }
    // The `type` field also covers the leading `:` token, so pick the first
    // named child of that field.
    let mut cursor = node.walk();
    if let Some(ty) = node
        .children_by_field_name("type", &mut cursor)
        .find(|c| c.is_named())
    {
        return format!("instance {}", node_text(&ty, source));
    }
    "(instance)".to_string()
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
