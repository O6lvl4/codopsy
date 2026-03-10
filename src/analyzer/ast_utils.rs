use tree_sitter::{Language, Node, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    TypeScript,
    Tsx,
    JavaScript,
    Rust,
}

impl SourceLanguage {
    pub fn is_js_ts(&self) -> bool {
        matches!(
            self,
            SourceLanguage::TypeScript | SourceLanguage::Tsx | SourceLanguage::JavaScript
        )
    }

    pub fn is_rust(&self) -> bool {
        matches!(self, SourceLanguage::Rust)
    }
}

pub fn get_language(file_path: &str) -> SourceLanguage {
    if file_path.ends_with(".tsx") {
        SourceLanguage::Tsx
    } else if file_path.ends_with(".ts") {
        SourceLanguage::TypeScript
    } else if file_path.ends_with(".rs") {
        SourceLanguage::Rust
    } else {
        SourceLanguage::JavaScript
    }
}

fn get_ts_language(lang: &SourceLanguage) -> Language {
    match lang {
        SourceLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        SourceLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        SourceLanguage::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        SourceLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
    }
}

pub fn parse_source(source: &str, language: SourceLanguage) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_lang = get_ts_language(&language);
    if parser.set_language(&ts_lang).is_err() {
        eprintln!("Failed to set language for {:?}", language);
        return None;
    }
    parser.parse(source.as_bytes(), None)
}

/// Check if a node is a function definition.
/// Works for both JS/TS and Rust node types.
pub fn is_function_node(node: &Node) -> bool {
    matches!(
        node.kind(),
        // JS/TS
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
    )
}

/// Get a human-readable function name from a function node.
pub fn get_function_name<'a>(node: &Node<'a>, source: &'a [u8]) -> String {
    let kind = node.kind();

    match kind {
        "function_item" | "function_declaration" | "generator_function_declaration" => {
            name_field_or(node, source, "(anonymous)")
        }
        "closure_expression" => name_from_closure(node, source),
        "method_definition" => name_from_method(node, source),
        "arrow_function" | "function_expression" | "function" | "generator_function" => {
            name_from_expr(node, source)
        }
        _ => "(anonymous)".to_string(),
    }
}

fn name_field_or(node: &Node, source: &[u8], fallback: &str) -> String {
    node.child_by_field_name("name")
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
            "variable_declarator" => "name",
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
