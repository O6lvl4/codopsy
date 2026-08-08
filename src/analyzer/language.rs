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
    Lean,
}

/// Everything that varies per language, in one row per language.
///
/// This used to be three lists that had to stay in lockstep — a name array
/// indexed by the enum's declaration order, a grammar `match`, and an extension
/// map — so adding a language meant editing all three and silently renaming
/// every later language if the order slipped.
pub struct LanguageSpec {
    pub lang: SourceLanguage,
    pub name: &'static str,
    /// Lowercase extensions, without the leading dot.
    pub extensions: &'static [&'static str],
    grammar: fn() -> Language,
}

const LANGUAGES: &[LanguageSpec] = &[
    spec(SourceLanguage::Tsx, "TSX", &["tsx"], || {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }),
    spec(SourceLanguage::TypeScript, "TypeScript", &["ts"], || {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }),
    spec(
        SourceLanguage::JavaScript,
        "JavaScript",
        &["js", "jsx", "mjs", "cjs"],
        || tree_sitter_javascript::LANGUAGE.into(),
    ),
    spec(SourceLanguage::Rust, "Rust", &["rs"], || {
        tree_sitter_rust::LANGUAGE.into()
    }),
    spec(SourceLanguage::Go, "Go", &["go"], || {
        tree_sitter_go::LANGUAGE.into()
    }),
    spec(SourceLanguage::Python, "Python", &["py", "pyi"], || {
        tree_sitter_python::LANGUAGE.into()
    }),
    spec(
        SourceLanguage::Cpp,
        "C++",
        &["cpp", "cc", "cxx", "hpp", "hxx"],
        || tree_sitter_cpp::LANGUAGE.into(),
    ),
    spec(SourceLanguage::C, "C", &["c", "h"], || {
        tree_sitter_c::LANGUAGE.into()
    }),
    spec(SourceLanguage::Java, "Java", &["java"], || {
        tree_sitter_java::LANGUAGE.into()
    }),
    spec(SourceLanguage::Ruby, "Ruby", &["rb"], || {
        tree_sitter_ruby::LANGUAGE.into()
    }),
    spec(SourceLanguage::CSharp, "C#", &["cs"], || {
        tree_sitter_c_sharp::LANGUAGE.into()
    }),
    spec(SourceLanguage::Php, "PHP", &["php"], || {
        tree_sitter_php::LANGUAGE_PHP.into()
    }),
    spec(SourceLanguage::Scala, "Scala", &["scala", "sc"], || {
        tree_sitter_scala::LANGUAGE.into()
    }),
    spec(SourceLanguage::Haskell, "Haskell", &["hs"], || {
        tree_sitter_haskell::LANGUAGE.into()
    }),
    spec(SourceLanguage::Bash, "Bash", &["sh", "bash", "zsh"], || {
        tree_sitter_bash::LANGUAGE.into()
    }),
    spec(SourceLanguage::Html, "HTML", &["html", "htm"], || {
        tree_sitter_html::LANGUAGE.into()
    }),
    spec(SourceLanguage::Css, "CSS", &["css"], || {
        tree_sitter_css::LANGUAGE.into()
    }),
    spec(SourceLanguage::Json, "JSON", &["json"], || {
        tree_sitter_json::LANGUAGE.into()
    }),
    spec(SourceLanguage::OCaml, "OCaml", &["ml", "mli"], || {
        tree_sitter_ocaml::LANGUAGE_OCAML.into()
    }),
    spec(SourceLanguage::Swift, "Swift", &["swift"], || {
        tree_sitter_swift::LANGUAGE.into()
    }),
    spec(SourceLanguage::Lua, "Lua", &["lua"], || {
        tree_sitter_lua::LANGUAGE.into()
    }),
    spec(SourceLanguage::Zig, "Zig", &["zig"], || {
        tree_sitter_zig::LANGUAGE.into()
    }),
    spec(SourceLanguage::Elixir, "Elixir", &["ex", "exs"], || {
        tree_sitter_elixir::LANGUAGE.into()
    }),
    spec(SourceLanguage::Yaml, "YAML", &["yml", "yaml"], || {
        tree_sitter_yaml::LANGUAGE.into()
    }),
    spec(SourceLanguage::Almide, "Almide", &["almd"], || {
        tree_sitter_almide::LANGUAGE.into()
    }),
    spec(
        SourceLanguage::Clojure,
        "Clojure",
        &["clj", "cljs", "cljc", "edn"],
        || tree_sitter_clojure::LANGUAGE.into(),
    ),
    spec(SourceLanguage::Erlang, "Erlang", &["erl", "hrl"], || {
        tree_sitter_erlang::LANGUAGE.into()
    }),
    spec(SourceLanguage::Gleam, "Gleam", &["gleam"], || {
        tree_sitter_gleam::LANGUAGE.into()
    }),
    spec(SourceLanguage::Kotlin, "Kotlin", &["kt", "kts"], || {
        tree_sitter_kotlin::LANGUAGE.into()
    }),
    spec(SourceLanguage::Crystal, "Crystal", &["cr"], || {
        tree_sitter_crystal::LANGUAGE.into()
    }),
    spec(SourceLanguage::Dart, "Dart", &["dart"], || {
        tree_sitter_dart::LANGUAGE.into()
    }),
    spec(SourceLanguage::Elm, "Elm", &["elm"], || {
        tree_sitter_elm::LANGUAGE.into()
    }),
    spec(
        SourceLanguage::Groovy,
        "Groovy",
        &["groovy", "gvy", "gy", "gsh"],
        || tree_sitter_groovy::LANGUAGE.into(),
    ),
    spec(SourceLanguage::Julia, "Julia", &["jl"], || {
        tree_sitter_julia::LANGUAGE.into()
    }),
    spec(SourceLanguage::Lean, "Lean 4", &["lean"], || {
        tree_sitter_lean::LANGUAGE.into()
    }),
];

const fn spec(
    lang: SourceLanguage,
    name: &'static str,
    extensions: &'static [&'static str],
    grammar: fn() -> Language,
) -> LanguageSpec {
    LanguageSpec { lang, name, extensions, grammar }
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

    fn spec(&self) -> &'static LanguageSpec {
        LANGUAGES
            .iter()
            .find(|s| s.lang == *self)
            .expect("every SourceLanguage needs a row in LANGUAGES")
    }

    pub fn name(&self) -> &'static str {
        self.spec().name
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        self.spec().extensions
    }

    pub fn tree_sitter_language(&self) -> Language {
        (self.spec().grammar)()
    }
}

/// Every language codopsy knows about.
pub fn all_languages() -> impl Iterator<Item = SourceLanguage> {
    LANGUAGES.iter().map(|s| s.lang)
}

pub fn get_language(file_path: &str) -> Option<SourceLanguage> {
    let path = file_path.to_lowercase();
    if path.ends_with(".d.ts") || path.ends_with(".d.tsx") {
        return None;
    }
    let ext = path.rsplit('.').next()?;
    LANGUAGES
        .iter()
        .find(|s| s.extensions.contains(&ext))
        .map(|s| s.lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_grammar_loads_and_round_trips() {
        for spec in LANGUAGES {
            let lang = spec.lang;
            assert_eq!(lang.name(), spec.name);
            let mut parser = tree_sitter::Parser::new();
            parser
                .set_language(&lang.tree_sitter_language())
                .unwrap_or_else(|e| panic!("grammar for {} failed to load: {e}", spec.name));
            for ext in spec.extensions {
                assert_eq!(
                    get_language(&format!("file.{ext}")),
                    Some(lang),
                    "extension .{ext} does not resolve to {}",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn names_and_extensions_are_unique() {
        let mut names = HashSet::new();
        let mut extensions = HashSet::new();
        for spec in LANGUAGES {
            assert!(names.insert(spec.name), "duplicate name {}", spec.name);
            for ext in spec.extensions {
                assert!(extensions.insert(*ext), "extension .{ext} claimed twice");
            }
        }
        assert_eq!(all_languages().count(), LANGUAGES.len());
    }

    #[test]
    fn type_declaration_files_are_skipped() {
        assert_eq!(get_language("types.d.ts"), None);
        assert_eq!(get_language("types.ts"), Some(SourceLanguage::TypeScript));
    }
}
