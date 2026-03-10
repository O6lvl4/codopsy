use tree_sitter::{Language, Node, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    TypeScript,
    Tsx,
    JavaScript,
    Rust,
    Go,
    Python,
    C,
    Cpp,
    Java,
    Ruby,
    CSharp,
    Php,
    Scala,
    Haskell,
    Bash,
    Html,
    Css,
    Json,
    OCaml,
    Swift,
    Lua,
    Zig,
    Elixir,
    Yaml,
    Almide,
}

impl SourceLanguage {
    pub fn is_js_ts(&self) -> bool {
        matches!(self, Self::TypeScript | Self::Tsx | Self::JavaScript)
    }

    pub fn is_rust(&self) -> bool {
        matches!(self, Self::Rust)
    }

    pub fn is_c_family(&self) -> bool {
        matches!(self, Self::C | Self::Cpp | Self::CSharp | Self::Java)
    }

    pub fn is_markup_or_data(&self) -> bool {
        matches!(self, Self::Html | Self::Css | Self::Json | Self::Yaml)
    }

    pub fn name(&self) -> &'static str {
        LANGUAGE_NAMES[*self as usize]
    }

    pub fn tree_sitter_language(&self) -> Language {
        use SourceLanguage::*;
        match self {
            TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Rust => tree_sitter_rust::LANGUAGE.into(),
            Go => tree_sitter_go::LANGUAGE.into(),
            Python => tree_sitter_python::LANGUAGE.into(),
            C => tree_sitter_c::LANGUAGE.into(),
            Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Java => tree_sitter_java::LANGUAGE.into(),
            Ruby => tree_sitter_ruby::LANGUAGE.into(),
            CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Php => tree_sitter_php::LANGUAGE_PHP.into(),
            Scala => tree_sitter_scala::LANGUAGE.into(),
            Haskell => tree_sitter_haskell::LANGUAGE.into(),
            Bash => tree_sitter_bash::LANGUAGE.into(),
            Html => tree_sitter_html::LANGUAGE.into(),
            Css => tree_sitter_css::LANGUAGE.into(),
            Json => tree_sitter_json::LANGUAGE.into(),
            OCaml => tree_sitter_ocaml::LANGUAGE_OCAML.into(),
            Swift => tree_sitter_swift::LANGUAGE.into(),
            Lua => tree_sitter_lua::LANGUAGE.into(),
            Zig => tree_sitter_zig::LANGUAGE.into(),
            Elixir => tree_sitter_elixir::LANGUAGE.into(),
            Yaml => tree_sitter_yaml::LANGUAGE.into(),
            Almide => tree_sitter_almide::LANGUAGE.into(),
        }
    }
}

const LANGUAGE_NAMES: &[&str] = &[
    "TypeScript", "TSX", "JavaScript", "Rust", "Go", "Python",
    "C", "C++", "Java", "Ruby", "C#", "PHP", "Scala", "Haskell",
    "Bash", "HTML", "CSS", "JSON", "OCaml",
    "Swift", "Lua", "Zig", "Elixir", "YAML", "Almide",
];

pub fn get_language(file_path: &str) -> Option<SourceLanguage> {
    // Handle multi-part extensions first
    let path = file_path.to_lowercase();
    if path.ends_with(".d.ts") || path.ends_with(".d.tsx") {
        return None; // Declaration files, skip
    }

    let ext = path.rsplit('.').next()?;
    EXT_LANGUAGE_MAP.iter()
        .find(|(exts, _)| exts.contains(&ext))
        .map(|(_, lang)| *lang)
}

const EXT_LANGUAGE_MAP: &[(&[&str], SourceLanguage)] = &[
    (&["tsx"], SourceLanguage::Tsx),
    (&["ts"], SourceLanguage::TypeScript),
    (&["js", "jsx", "mjs", "cjs"], SourceLanguage::JavaScript),
    (&["rs"], SourceLanguage::Rust),
    (&["go"], SourceLanguage::Go),
    (&["py", "pyi"], SourceLanguage::Python),
    (&["cpp", "cc", "cxx", "hpp", "hxx"], SourceLanguage::Cpp),
    (&["c", "h"], SourceLanguage::C),
    (&["java"], SourceLanguage::Java),
    (&["rb"], SourceLanguage::Ruby),
    (&["cs"], SourceLanguage::CSharp),
    (&["php"], SourceLanguage::Php),
    (&["scala", "sc"], SourceLanguage::Scala),
    (&["hs"], SourceLanguage::Haskell),
    (&["sh", "bash", "zsh"], SourceLanguage::Bash),
    (&["html", "htm"], SourceLanguage::Html),
    (&["css"], SourceLanguage::Css),
    (&["json"], SourceLanguage::Json),
    (&["ml", "mli"], SourceLanguage::OCaml),
    (&["swift"], SourceLanguage::Swift),
    (&["lua"], SourceLanguage::Lua),
    (&["zig"], SourceLanguage::Zig),
    (&["ex", "exs"], SourceLanguage::Elixir),
    (&["yml", "yaml"], SourceLanguage::Yaml),
    (&["almd"], SourceLanguage::Almide),
];

pub fn parse_source(source: &str, language: SourceLanguage) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_lang = language.tree_sitter_language();
    if parser.set_language(&ts_lang).is_err() {
        eprintln!("Failed to set language for {:?}", language);
        return None;
    }
    parser.parse(source.as_bytes(), None)
}

/// Check if a node is a function definition.
/// Works for JS/TS, Rust, Go, Python, C/C++, Java, Ruby, C#, PHP, Scala, Haskell, OCaml.
pub fn is_function_node(node: &Node) -> bool {
    matches!(
        node.kind(),
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
            | "do_block"
            // PHP
            | "anonymous_function_creation_expression"
            // OCaml
            | "let_binding"
            | "value_definition"
            // Almide
            | "test_declaration"
    )
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
        "do_block" => "(block)".to_string(),
        _ => "(anonymous)".to_string(),
    }
}

fn name_field_or(node: &Node, source: &[u8], fallback: &str) -> String {
    node.child_by_field_name("name")
        .or_else(|| find_child_by_kind(node, "function_name"))
        .map(|n| node_text(&n, source).to_string())
        .unwrap_or_else(|| fallback.to_string())
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
