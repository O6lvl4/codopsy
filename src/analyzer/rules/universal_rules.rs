use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{
    is_clojure_defn, is_elixir_def_call, is_erlang_fun_decl, is_function_node, node_text,
};
use crate::types::{Issue, Severity};

use super::run_check;

#[cfg(test)]
use crate::analyzer::ast_utils::{parse_source, SourceLanguage};

/// Generalized empty function check that works across all languages.
/// Detects function bodies that contain no statements (only braces/keywords and comments).
pub fn check_no_empty_function_universal(
    tree: &Tree,
    source: &[u8],
    fp: &str,
    sev: Severity,
) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let is_func = is_function_node(node)
            || is_elixir_def_call(node, ctx.source)
            || is_clojure_defn(node, ctx.source)
            || is_erlang_fun_decl(node);
        if !is_func {
            return;
        }
        if has_empty_body(node) {
            ctx.report(
                node,
                "no-empty-function",
                "Unexpected empty function".into(),
            );
        }
    })
}

/// Check if a function node has an empty body.
/// Handles multiple strategies:
/// 1. Languages with a "body" field (Python, C, C++, Java, Go, Rust, Swift, etc.)
/// 2. Languages with a block-type child (JS/TS: statement_block)
/// 3. "Flat" function nodes (Ruby method, Lua function_declaration) where
///    the body is inline among children with no separate block node.
fn has_empty_body(node: &Node) -> bool {
    // Strategy 1: explicit "body" field
    if let Some(body) = node.child_by_field_name("body") {
        return is_empty_block(&body);
    }

    // Strategy 2: search for a block-type child
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    for child in &children {
        let kind = child.kind();
        if matches!(
            kind,
            "block"
                | "statement_block"
                | "compound_statement"
                | "do_block"
                | "code_block"
                | "function_body"
                | "chunk"
        ) {
            return is_empty_block(child);
        }
    }

    // Strategy 3: flat structure (Ruby, Lua).
    // If all children are structural tokens, keywords, name, or parameters → empty.
    let has_statement_child = children.iter().any(|c| {
        let k = c.kind();
        !is_structural_token(k)
            && !is_function_keyword(k)
            && !matches!(k, "identifier" | "simple_identifier" | "parameters"
                | "formal_parameters" | "type_annotation" | "return_type"
                | ":" | "->" | "=>" | "scope_resolution" | "field_identifier"
                | "name" | "function_name" | "visibility_modifier" | "pub"
                | "func" | "function" | "def" | "fn")
    });
    !has_statement_child
}

/// Keywords used in function syntax that are not statements.
fn is_function_keyword(kind: &str) -> bool {
    matches!(
        kind,
        "function" | "func" | "def" | "fn" | "pub" | "async" | "static"
            | "override" | "virtual" | "abstract" | "private" | "protected"
            | "public" | "final" | "const" | "export" | "default"
    )
}

/// Check for TODO/FIXME/HACK/XXX comments in source code.
pub fn check_todo_comments(
    tree: &Tree,
    source: &[u8],
    fp: &str,
    sev: Severity,
) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let kind = node.kind();
        if !matches!(
            kind,
            "comment" | "line_comment" | "block_comment" | "hash_comment"
        ) {
            return;
        }
        let text = node_text(node, ctx.source);
        for marker in &["TODO", "FIXME", "HACK", "XXX"] {
            if text.contains(marker) {
                ctx.report(
                    node,
                    "todo-comment",
                    format!("{marker} comment found"),
                );
                return;
            }
        }
    })
}

/// Check if a block node contains no meaningful statements.
fn is_empty_block(block: &Node) -> bool {
    let kind = block.kind();
    if !matches!(
        kind,
        "block"
            | "statement_block"
            | "compound_statement"
            | "class_body"
            | "body"
            | "do_block"
            // Swift
            | "function_body"
            | "code_block"
            // Lua
            | "chunk"
            // Elixir
            | "keyword_list"
    ) {
        return false;
    }
    let mut cursor = block.walk();
    let has_statements = block.children(&mut cursor).any(|c| {
        let k = c.kind();
        !is_structural_token(k)
    });
    !has_statements
}

/// Tokens that are structural/syntactic and do not count as "statements".
fn is_structural_token(kind: &str) -> bool {
    matches!(
        kind,
        "{" | "}"
            | "comment"
            | "line_comment"
            | "block_comment"
            | "hash_comment"
            | "("
            | ")"
            | ","
            | "do"
            | "end"
            | "begin"
            | "indent"
            | "dedent"
            | "newline"
            | "NEWLINE"
            | "INDENT"
            | "DEDENT"
            | "then"
            | "keyword_pair"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_empty_fn(src: &str, lang: SourceLanguage) -> bool {
        let tree = parse_source(src, lang).unwrap();
        let issues = check_no_empty_function_universal(&tree, src.as_bytes(), "test", Severity::Warning);
        issues.iter().any(|i| i.rule == "no-empty-function")
    }

    fn check_todo(src: &str, lang: SourceLanguage) -> bool {
        let tree = parse_source(src, lang).unwrap();
        let issues = check_todo_comments(&tree, src.as_bytes(), "test", Severity::Info);
        issues.iter().any(|i| i.rule == "todo-comment")
    }

    #[test]
    fn empty_function_go() {
        assert!(check_empty_fn("package main\nfunc empty() {}", SourceLanguage::Go));
        assert!(!check_empty_fn("package main\nfunc notempty() { return }", SourceLanguage::Go));
    }

    #[test]
    fn empty_function_ruby() {
        assert!(check_empty_fn("def empty\nend", SourceLanguage::Ruby));
    }

    #[test]
    fn empty_function_lua() {
        assert!(check_empty_fn("function empty()\nend", SourceLanguage::Lua));
    }

    #[test]
    fn empty_function_swift() {
        assert!(check_empty_fn("func empty() {}", SourceLanguage::Swift));
    }

    #[test]
    fn empty_function_java() {
        assert!(check_empty_fn("class T { void empty() {} }", SourceLanguage::Java));
    }

    #[test]
    fn empty_function_cpp() {
        assert!(check_empty_fn("void empty() {}", SourceLanguage::Cpp));
    }

    #[test]
    fn empty_function_elixir() {
        assert!(check_empty_fn("def empty do\nend", SourceLanguage::Elixir));
    }

    #[test]
    fn python_pass_is_not_empty() {
        assert!(!check_empty_fn("def foo():\n    pass", SourceLanguage::Python));
    }

    #[test]
    fn todo_comment_various_languages() {
        assert!(check_todo("// TODO: fix", SourceLanguage::JavaScript));
        assert!(check_todo("# TODO: fix", SourceLanguage::Python));
        assert!(check_todo("# FIXME: fix", SourceLanguage::Ruby));
        assert!(check_todo("// HACK: fix", SourceLanguage::Rust));
    }
}
