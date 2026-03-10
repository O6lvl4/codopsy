use tree_sitter::Tree;

use crate::analyzer::ast_utils::SourceLanguage;
use crate::config::CodopsyConfig;
use crate::types::{Issue, Severity};

use super::rules::bug_detection::*;
use super::rules::control_flow::*;
use super::rules::rust_rules::*;
use super::rules::style_rules::*;
use super::rules::threshold_rules::*;

/// Default thresholds
const DEFAULT_MAX_LINES: usize = 300;
const DEFAULT_MAX_DEPTH: usize = 4;
const DEFAULT_MAX_PARAMS: usize = 4;

type SimpleCheckFn = fn(&Tree, &[u8], &str, Severity) -> Vec<Issue>;

/// Rules that only apply to JS/TS files.
const JS_TS_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-any", Severity::Warning, check_no_any),
    ("no-console", Severity::Warning, check_no_console),
    ("no-var", Severity::Warning, check_no_var),
    ("eqeqeq", Severity::Warning, check_eqeqeq),
    ("no-empty-function", Severity::Warning, check_no_empty_function),
    ("no-nested-ternary", Severity::Warning, check_no_nested_ternary),
    ("no-debugger", Severity::Error, check_no_debugger),
    ("no-duplicate-case", Severity::Error, check_no_duplicate_case),
    ("no-self-assign", Severity::Warning, check_no_self_assign),
    ("no-eval", Severity::Error, check_no_eval),
    ("no-unreachable", Severity::Error, check_no_unreachable),
];

/// Rules that only apply to Rust files.
const RUST_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-unsafe", Severity::Warning, check_no_unsafe),
    ("no-unwrap", Severity::Warning, check_no_unwrap),
    ("no-dbg", Severity::Warning, check_no_dbg),
    ("no-todo", Severity::Warning, check_no_todo),
    ("no-println", Severity::Info, check_no_println),
    (
        "no-empty-function",
        Severity::Warning,
        check_no_empty_function_rust,
    ),
];

/// Rules that apply to all languages.
const UNIVERSAL_RULES: &[(&str, Severity, SimpleCheckFn)] = &[];

pub fn lint_file(
    file_path: &str,
    source: &str,
    tree: &Tree,
    config: &CodopsyConfig,
    language: SourceLanguage,
) -> Vec<Issue> {
    let source_bytes = source.as_bytes();
    let mut issues = Vec::new();

    // Run universal rules
    run_simple_rules(UNIVERSAL_RULES, tree, source_bytes, file_path, config, &mut issues);

    // Run language-specific rules
    if language.is_js_ts() {
        run_simple_rules(JS_TS_RULES, tree, source_bytes, file_path, config, &mut issues);
    } else if language.is_rust() {
        run_simple_rules(RUST_RULES, tree, source_bytes, file_path, config, &mut issues);
    }

    // Threshold rules (language-agnostic)
    run_threshold_rules(tree, source_bytes, file_path, config, &mut issues, language);

    issues
}

fn run_simple_rules(
    rules: &[(&str, Severity, SimpleCheckFn)],
    tree: &Tree,
    source_bytes: &[u8],
    file_path: &str,
    config: &CodopsyConfig,
    issues: &mut Vec<Issue>,
) {
    for &(name, default_severity, check_fn) in rules {
        if config.is_rule_disabled(name) {
            continue;
        }
        let severity = config.get_rule_severity(name).unwrap_or(default_severity);
        issues.extend(check_fn(tree, source_bytes, file_path, severity));
    }
}

fn run_threshold_rules(
    tree: &Tree,
    source_bytes: &[u8],
    file_path: &str,
    config: &CodopsyConfig,
    issues: &mut Vec<Issue>,
    language: SourceLanguage,
) {
    if !config.is_rule_disabled("max-lines") {
        let severity = config
            .get_rule_severity("max-lines")
            .unwrap_or(Severity::Warning);
        let max = config.get_rule_max("max-lines").unwrap_or(DEFAULT_MAX_LINES);
        issues.extend(check_max_lines(tree, source_bytes, file_path, severity, max));
    }

    if !config.is_rule_disabled("max-depth") {
        let severity = config
            .get_rule_severity("max-depth")
            .unwrap_or(Severity::Warning);
        let max = config.get_rule_max("max-depth").unwrap_or(DEFAULT_MAX_DEPTH);
        issues.extend(check_max_depth_for_language(
            tree,
            source_bytes,
            file_path,
            severity,
            max,
            language,
        ));
    }

    if !config.is_rule_disabled("max-params") {
        let severity = config
            .get_rule_severity("max-params")
            .unwrap_or(Severity::Warning);
        let max = config
            .get_rule_max("max-params")
            .unwrap_or(DEFAULT_MAX_PARAMS);
        issues.extend(check_max_params(tree, source_bytes, file_path, severity, max));
    }
}
