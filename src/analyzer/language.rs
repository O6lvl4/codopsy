use tree_sitter::Language;

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
    Clojure,
    Erlang,
    Gleam,
    Kotlin,
    Crystal,
    Dart,
    Elm,
    Groovy,
    Julia,
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

    pub fn is_beam(&self) -> bool {
        matches!(self, Self::Elixir | Self::Erlang | Self::Gleam)
    }

    pub fn is_lisp(&self) -> bool {
        matches!(self, Self::Clojure)
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
            Clojure => tree_sitter_clojure::LANGUAGE.into(),
            Erlang => tree_sitter_erlang::LANGUAGE.into(),
            Gleam => tree_sitter_gleam::LANGUAGE.into(),
            Kotlin => tree_sitter_kotlin::LANGUAGE.into(),
            Crystal => tree_sitter_crystal::LANGUAGE.into(),
            Dart => tree_sitter_dart::LANGUAGE.into(),
            Elm => tree_sitter_elm::LANGUAGE.into(),
            Groovy => tree_sitter_groovy::LANGUAGE.into(),
            Julia => tree_sitter_julia::LANGUAGE.into(),
        }
    }
}

const LANGUAGE_NAMES: &[&str] = &[
    "TypeScript", "TSX", "JavaScript", "Rust", "Go", "Python",
    "C", "C++", "Java", "Ruby", "C#", "PHP", "Scala", "Haskell",
    "Bash", "HTML", "CSS", "JSON", "OCaml",
    "Swift", "Lua", "Zig", "Elixir", "YAML", "Almide",
    "Clojure", "Erlang", "Gleam",
    "Kotlin", "Crystal", "Dart", "Elm", "Groovy", "Julia",
];

pub fn get_language(file_path: &str) -> Option<SourceLanguage> {
    let path = file_path.to_lowercase();
    if path.ends_with(".d.ts") || path.ends_with(".d.tsx") {
        return None;
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
    (&["clj", "cljs", "cljc", "edn"], SourceLanguage::Clojure),
    (&["erl", "hrl"], SourceLanguage::Erlang),
    (&["gleam"], SourceLanguage::Gleam),
    (&["kt", "kts"], SourceLanguage::Kotlin),
    (&["cr"], SourceLanguage::Crystal),
    (&["dart"], SourceLanguage::Dart),
    (&["elm"], SourceLanguage::Elm),
    (&["groovy", "gvy", "gy", "gsh"], SourceLanguage::Groovy),
    (&["jl"], SourceLanguage::Julia),
];
