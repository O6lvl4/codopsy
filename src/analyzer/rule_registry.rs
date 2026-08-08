//! Selecting the rule set that applies to a file.

use tree_sitter::Tree;

use crate::analyzer::ast_utils::SourceLanguage;
use crate::types::{Issue, Severity};

use super::rule_tables::*;
use super::rules::universal_rules::{check_syntax_errors, check_todo_comments};

pub type SimpleCheckFn = fn(&Tree, &[u8], &str, Severity) -> Vec<Issue>;

pub const UNIVERSAL_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("todo-comment", Severity::Info, check_todo_comments),
    ("syntax-error", Severity::Info, check_syntax_errors),
];
/// Languages whose rule set is selected by the language alone. Families that
/// share a rule set (JS/TS, C/C++) are handled in [`language_rules`].
const RULES_BY_LANGUAGE: &[(SourceLanguage, &[(&str, Severity, SimpleCheckFn)])] = &[
    (SourceLanguage::Go, GO_RULES),
    (SourceLanguage::Python, PYTHON_RULES),
    (SourceLanguage::Java, JAVA_RULES),
    (SourceLanguage::Elixir, ELIXIR_RULES),
    (SourceLanguage::Erlang, ERLANG_RULES),
    (SourceLanguage::Gleam, GLEAM_RULES),
    (SourceLanguage::Clojure, CLOJURE_RULES),
    (SourceLanguage::Ruby, RUBY_RULES),
    (SourceLanguage::Php, PHP_RULES),
    (SourceLanguage::Lua, LUA_RULES),
    (SourceLanguage::Swift, SWIFT_RULES),
    (SourceLanguage::Zig, ZIG_RULES),
    (SourceLanguage::Haskell, HASKELL_RULES),
    (SourceLanguage::Scala, SCALA_RULES),
    (SourceLanguage::Kotlin, KOTLIN_RULES),
    (SourceLanguage::Crystal, CRYSTAL_RULES),
    (SourceLanguage::Dart, DART_RULES),
    (SourceLanguage::Elm, ELM_RULES),
    (SourceLanguage::Groovy, GROOVY_RULES),
    (SourceLanguage::Julia, JULIA_RULES),
    (SourceLanguage::Lean, LEAN_RULES),
    (SourceLanguage::Bash, BASH_RULES),
];
pub fn language_rules(lang: SourceLanguage) -> &'static [(&'static str, Severity, SimpleCheckFn)] {
    if lang.is_js_ts() { return JS_TS_RULES; }
    if lang.is_rust() { return RUST_RULES; }
    if matches!(lang, SourceLanguage::C | SourceLanguage::Cpp) { return C_CPP_RULES; }
    RULES_BY_LANGUAGE
        .iter()
        .find(|(l, _)| *l == lang)
        .map_or(&[], |(_, rules)| rules)
}
