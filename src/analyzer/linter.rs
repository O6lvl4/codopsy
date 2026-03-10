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

struct LintCtx<'a> {
    tree: &'a Tree,
    source_bytes: &'a [u8],
    file_path: &'a str,
    config: &'a CodopsyConfig,
    issues: Vec<Issue>,
}

impl<'a> LintCtx<'a> {
    fn run_rules(&mut self, rules: &[(&str, Severity, SimpleCheckFn)]) {
        for &(name, default_severity, check_fn) in rules {
            if self.config.is_rule_disabled(name) {
                continue;
            }
            let severity = self.config.get_rule_severity(name).unwrap_or(default_severity);
            self.issues.extend(check_fn(self.tree, self.source_bytes, self.file_path, severity));
        }
    }

    fn run_threshold_rules(&mut self, language: SourceLanguage) {
        self.run_threshold("max-lines", DEFAULT_MAX_LINES, |t, s, f, sev, max| {
            check_max_lines(t, s, f, sev, max)
        });
        self.run_threshold("max-depth", DEFAULT_MAX_DEPTH, |t, s, f, sev, max| {
            check_max_depth_for_language(t, s, f, sev, max, language)
        });
        self.run_threshold("max-params", DEFAULT_MAX_PARAMS, |t, s, f, sev, max| {
            check_max_params(t, s, f, sev, max)
        });
    }

    fn run_threshold(
        &mut self,
        name: &str,
        default_max: usize,
        check: impl FnOnce(&Tree, &[u8], &str, Severity, usize) -> Vec<Issue>,
    ) {
        if self.config.is_rule_disabled(name) {
            return;
        }
        let severity = self.config.get_rule_severity(name).unwrap_or(Severity::Warning);
        let max = self.config.get_rule_max(name).unwrap_or(default_max);
        self.issues.extend(check(self.tree, self.source_bytes, self.file_path, severity, max));
    }
}

pub fn lint_file(
    file_path: &str,
    source: &str,
    tree: &Tree,
    config: &CodopsyConfig,
    language: SourceLanguage,
) -> Vec<Issue> {
    let mut ctx = LintCtx {
        tree,
        source_bytes: source.as_bytes(),
        file_path,
        config,
        issues: Vec::new(),
    };

    ctx.run_rules(UNIVERSAL_RULES);

    if language.is_js_ts() {
        ctx.run_rules(JS_TS_RULES);
    } else if language.is_rust() {
        ctx.run_rules(RUST_RULES);
    }

    ctx.run_threshold_rules(language);

    ctx.issues
}
