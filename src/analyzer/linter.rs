use tree_sitter::Tree;

use crate::analyzer::ast_utils::SourceLanguage;
use crate::config::CodopsyConfig;
use crate::defaults;
use crate::types::{Issue, Severity};

use super::rules::bug_detection::*;
use super::rules::c_rules::*;
use super::rules::clojure_rules;
use super::rules::control_flow::*;
use super::rules::elixir_rules::*;
use super::rules::erlang_rules::*;
use super::rules::gleam_rules;
use super::rules::go_rules;
use super::rules::java_rules::*;
use super::rules::python_rules::*;
use super::rules::rust_rules;
use super::rules::style_rules::*;
use super::rules::threshold_rules::*;
use super::rules::universal_rules::*;

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
    ("no-constant-condition", Severity::Warning, check_no_constant_condition),
    ("default-case", Severity::Warning, check_no_missing_default),
    ("no-fallthrough", Severity::Warning, check_no_fallthrough),
    ("no-self-compare", Severity::Warning, check_no_self_compare),
    ("no-useless-catch", Severity::Error, check_no_useless_catch),
    ("use-isnan", Severity::Error, check_use_isnan),
    ("no-compare-neg-zero", Severity::Error, check_no_compare_neg_zero),
    ("no-unsafe-negation", Severity::Error, check_no_unsafe_negation),
    ("no-constructor-return", Severity::Error, check_no_constructor_return),
    ("valid-typeof", Severity::Error, check_valid_typeof),
    ("no-useless-rename", Severity::Warning, check_no_useless_rename),
    ("no-empty-pattern", Severity::Warning, check_no_empty_pattern),
];

/// Rules that only apply to Rust files.
const RUST_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-unsafe", Severity::Warning, rust_rules::check_no_unsafe),
    ("no-unwrap", Severity::Warning, rust_rules::check_no_unwrap),
    ("no-dbg", Severity::Warning, rust_rules::check_no_dbg),
    ("no-todo", Severity::Warning, rust_rules::check_no_todo),
    ("no-println", Severity::Info, rust_rules::check_no_println),
    ("needless-bool", Severity::Warning, rust_rules::check_needless_bool),
    (
        "no-empty-function",
        Severity::Warning,
        rust_rules::check_no_empty_function_rust,
    ),
    ("needless-return", Severity::Warning, rust_rules::check_needless_return),
    ("bool-comparison", Severity::Warning, rust_rules::check_bool_comparison),
    ("collapsible-if", Severity::Warning, rust_rules::check_collapsible_if),
    ("single-match", Severity::Warning, rust_rules::check_single_match),
    ("manual-map", Severity::Warning, rust_rules::check_manual_map),
    ("redundant-clone", Severity::Warning, rust_rules::check_redundant_clone),
    ("eq-op", Severity::Warning, rust_rules::check_eq_op),
];

/// Rules that only apply to Go files.
const GO_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-panic", Severity::Warning, go_rules::check_no_panic),
    ("no-fmt-print", Severity::Info, go_rules::check_no_fmt_print),
    ("no-ignored-error", Severity::Warning, go_rules::check_no_ignored_error),
    ("no-os-exit", Severity::Warning, go_rules::check_no_os_exit),
    ("no-defer-in-loop", Severity::Warning, go_rules::check_no_defer_in_loop),
    ("no-empty-block", Severity::Warning, go_rules::check_no_empty_block),
    ("no-unreachable", Severity::Error, go_rules::check_no_unreachable_go),
    ("no-naked-return", Severity::Warning, go_rules::check_no_naked_return),
    ("no-range-over-string", Severity::Info, go_rules::check_no_range_over_string),
    ("no-shadow-import", Severity::Warning, go_rules::check_no_shadow_import),
];

/// Rules that only apply to Python files.
const PYTHON_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-bare-except", Severity::Warning, check_no_bare_except),
    ("no-print", Severity::Info, check_no_print),
    ("no-eval", Severity::Error, check_no_eval_exec),
    ("no-mutable-default", Severity::Warning, check_no_mutable_default),
    ("no-global", Severity::Warning, check_no_global),
    ("no-assert", Severity::Info, check_no_assert),
    ("unreachable", Severity::Warning, check_unreachable),
    ("pointless-except", Severity::Warning, check_no_pointless_except),
    ("no-pass-body", Severity::Info, check_no_pass_body),
    ("no-star-import", Severity::Warning, check_no_star_import),
    ("no-nested-with", Severity::Warning, check_no_nested_with),
    ("no-return-in-init", Severity::Error, check_no_return_in_init),
    ("simplify-boolean-return", Severity::Warning, check_simplify_boolean_return),
];

/// Rules that only apply to Java files.
const JAVA_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-sysout", Severity::Warning, check_no_sysout),
    ("no-print-stack-trace", Severity::Warning, check_no_print_stack_trace),
    ("no-empty-catch", Severity::Warning, check_no_empty_catch),
    ("no-throws-exception", Severity::Warning, check_no_throws_exception),
    ("no-raw-type", Severity::Warning, check_no_raw_type),
    ("no-string-equality", Severity::Warning, check_no_string_equality),
    ("missing-switch-default", Severity::Warning, check_no_missing_switch_default),
];

/// Rules that apply to C and C++ files.
const C_CPP_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-printf", Severity::Info, check_no_printf),
    ("no-unsafe-fn", Severity::Error, check_no_unsafe_fn),
    ("no-malloc", Severity::Info, check_no_malloc),
    ("no-goto", Severity::Warning, check_no_goto),
];

/// Rules that only apply to Elixir files.
const ELIXIR_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-io-inspect", Severity::Warning, check_no_io_inspect),
    ("no-io-puts", Severity::Info, check_no_io_puts),
    ("no-raise-in-with", Severity::Warning, check_no_raise_in_with),
    ("pipe-into-anonymous", Severity::Warning, check_pipe_into_anonymous),
];

/// Rules that only apply to Erlang files.
const ERLANG_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-process-flag", Severity::Warning, check_no_process_flag),
    ("no-catch-all", Severity::Warning, check_no_catch_all),
    ("no-exit-call", Severity::Warning, check_no_exit_call),
];

/// Rules that only apply to Gleam files.
const GLEAM_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-todo", Severity::Warning, gleam_rules::check_no_todo),
    ("no-panic", Severity::Warning, gleam_rules::check_no_panic),
    ("no-let-assert", Severity::Warning, gleam_rules::check_no_let_assert),
];

/// Rules that only apply to Clojure files.
const CLOJURE_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, clojure_rules::check_no_println),
    ("no-def-in-def", Severity::Warning, clojure_rules::check_no_def_in_def),
    ("no-thread-sleep", Severity::Warning, clojure_rules::check_no_thread_sleep),
    ("no-reflection", Severity::Warning, clojure_rules::check_no_reflection),
];

/// Rules that apply to all languages with function bodies.
const UNIVERSAL_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("todo-comment", Severity::Info, check_todo_comments),
];

/// Languages that have their own empty-function check via language-specific rules.
fn has_dedicated_empty_function_rule(lang: SourceLanguage) -> bool {
    lang.is_js_ts() || lang.is_rust()
}

/// Languages that should get the universal empty-function check.
fn should_check_empty_function(lang: SourceLanguage) -> bool {
    !has_dedicated_empty_function_rule(lang) && !lang.is_markup_or_data()
}

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

    fn run_threshold_rules(&mut self) {
        self.run_threshold("max-lines", defaults::MAX_LINES, |t, s, f, sev, max| {
            check_max_lines(t, s, f, sev, max)
        });
        self.run_threshold("max-depth", defaults::MAX_DEPTH, |t, s, f, sev, max| {
            check_max_depth(t, s, f, sev, max)
        });
        self.run_threshold("max-params", defaults::MAX_PARAMS, |t, s, f, sev, max| {
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

    // Language-specific lint rules
    match language {
        _ if language.is_js_ts() => ctx.run_rules(JS_TS_RULES),
        _ if language.is_rust() => ctx.run_rules(RUST_RULES),
        SourceLanguage::Go => ctx.run_rules(GO_RULES),
        SourceLanguage::Python => ctx.run_rules(PYTHON_RULES),
        SourceLanguage::Java => ctx.run_rules(JAVA_RULES),
        SourceLanguage::C | SourceLanguage::Cpp => ctx.run_rules(C_CPP_RULES),
        SourceLanguage::Elixir => ctx.run_rules(ELIXIR_RULES),
        SourceLanguage::Erlang => ctx.run_rules(ERLANG_RULES),
        SourceLanguage::Gleam => ctx.run_rules(GLEAM_RULES),
        SourceLanguage::Clojure => ctx.run_rules(CLOJURE_RULES),
        _ => {}
    }

    // Universal empty-function for languages without a dedicated check
    if should_check_empty_function(language) && !config.is_rule_disabled("no-empty-function") {
        let severity = config.get_rule_severity("no-empty-function").unwrap_or(Severity::Warning);
        ctx.issues.extend(check_no_empty_function_universal(
            tree,
            source.as_bytes(),
            file_path,
            severity,
        ));
    }

    ctx.run_threshold_rules();

    ctx.issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ast_utils::parse_source;

    fn lint_js(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::JavaScript).unwrap();
        lint_file("test.js", source, &tree, &config, SourceLanguage::JavaScript)
    }

    fn lint_rs(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::Rust).unwrap();
        lint_file("test.rs", source, &tree, &config, SourceLanguage::Rust)
    }

    fn lint_py(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::Python).unwrap();
        lint_file("test.py", source, &tree, &config, SourceLanguage::Python)
    }

    fn lint_go(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::Go).unwrap();
        lint_file("test.go", source, &tree, &config, SourceLanguage::Go)
    }

    fn lint_java(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::Java).unwrap();
        lint_file("test.java", source, &tree, &config, SourceLanguage::Java)
    }

    fn lint_c(source: &str) -> Vec<Issue> {
        let config = CodopsyConfig::default();
        let tree = parse_source(source, SourceLanguage::C).unwrap();
        lint_file("test.c", source, &tree, &config, SourceLanguage::C)
    }

    // --- JS/TS ---
    #[test]
    fn detects_no_var() {
        let issues = lint_js("var x = 1;");
        assert!(issues.iter().any(|i| i.rule == "no-var"));
    }

    #[test]
    fn detects_eval() {
        let issues = lint_js("eval('code');");
        assert!(issues.iter().any(|i| i.rule == "no-eval"));
    }

    #[test]
    fn detects_debugger() {
        let issues = lint_js("debugger;");
        assert!(issues.iter().any(|i| i.rule == "no-debugger"));
    }

    #[test]
    fn detects_eqeqeq() {
        let issues = lint_js("if (x == 1) {}");
        assert!(issues.iter().any(|i| i.rule == "eqeqeq"));
    }

    #[test]
    fn no_issue_for_strict_eq() {
        let issues = lint_js("if (x === 1) {}");
        assert!(!issues.iter().any(|i| i.rule == "eqeqeq"));
    }

    #[test]
    fn detects_console_log() {
        let issues = lint_js("console.log('hello');");
        assert!(issues.iter().any(|i| i.rule == "no-console"));
    }

    // --- Rust ---
    #[test]
    fn detects_rust_unwrap() {
        let issues = lint_rs("fn main() { let x = Some(1).unwrap(); }");
        assert!(issues.iter().any(|i| i.rule == "no-unwrap"));
    }

    #[test]
    fn detects_rust_dbg() {
        let issues = lint_rs("fn main() { dbg!(42); }");
        assert!(issues.iter().any(|i| i.rule == "no-dbg"));
    }

    // --- Go ---
    #[test]
    fn detects_go_panic() {
        let issues = lint_go("package main\nfunc main() { panic(\"err\") }");
        assert!(issues.iter().any(|i| i.rule == "no-panic"));
    }

    #[test]
    fn detects_go_fmt_println() {
        let issues = lint_go("package main\nimport \"fmt\"\nfunc main() { fmt.Println(\"hi\") }");
        assert!(issues.iter().any(|i| i.rule == "no-fmt-print"));
    }

    #[test]
    fn detects_go_defer_in_loop() {
        let issues = lint_go("package main\nfunc f() { for i := 0; i < 10; i++ { defer close() } }");
        assert!(issues.iter().any(|i| i.rule == "no-defer-in-loop"));
    }

    #[test]
    fn detects_go_empty_if_block() {
        let issues = lint_go("package main\nfunc f() { if true {} }");
        assert!(issues.iter().any(|i| i.rule == "no-empty-block"));
    }

    #[test]
    fn detects_go_empty_for_block() {
        let issues = lint_go("package main\nfunc f() { for i := 0; i < 10; i++ {} }");
        assert!(issues.iter().any(|i| i.rule == "no-empty-block"));
    }

    #[test]
    fn no_empty_block_when_has_body() {
        let issues = lint_go("package main\nfunc f() { if true { x := 1; _ = x } }");
        assert!(!issues.iter().any(|i| i.rule == "no-empty-block"));
    }

    #[test]
    fn detects_go_unreachable() {
        let issues = lint_go("package main\nfunc f() int { return 1\nx := 2\n_ = x\nreturn x }");
        assert!(issues.iter().any(|i| i.rule == "no-unreachable"));
    }

    #[test]
    fn detects_go_naked_return() {
        let issues = lint_go("package main\nfunc f() (x int) { x = 1\nreturn }");
        assert!(issues.iter().any(|i| i.rule == "no-naked-return"));
    }

    #[test]
    fn no_naked_return_for_unnamed_results() {
        let issues = lint_go("package main\nfunc f() int { return 1 }");
        assert!(!issues.iter().any(|i| i.rule == "no-naked-return"));
    }

    #[test]
    fn detects_go_shadow_import() {
        let issues = lint_go("package main\nimport \"fmt\"\nfunc f() { fmt := 1\n_ = fmt }");
        assert!(issues.iter().any(|i| i.rule == "no-shadow-import"));
    }

    #[test]
    fn no_shadow_import_for_different_name() {
        let issues = lint_go("package main\nimport \"fmt\"\nfunc f() { x := 1\n_ = x }");
        assert!(!issues.iter().any(|i| i.rule == "no-shadow-import"));
    }

    // --- Python ---
    #[test]
    fn detects_python_bare_except() {
        let issues = lint_py("try:\n    pass\nexcept:\n    pass");
        assert!(issues.iter().any(|i| i.rule == "no-bare-except"));
    }

    #[test]
    fn detects_python_print() {
        let issues = lint_py("print('hello')");
        assert!(issues.iter().any(|i| i.rule == "no-print"));
    }

    #[test]
    fn detects_python_mutable_default() {
        let issues = lint_py("def foo(x=[]):\n    pass");
        assert!(issues.iter().any(|i| i.rule == "no-mutable-default"));
    }

    #[test]
    fn detects_python_global() {
        let issues = lint_py("def foo():\n    global x\n    x = 1");
        assert!(issues.iter().any(|i| i.rule == "no-global"));
    }

    #[test]
    fn detects_python_pointless_except() {
        let issues = lint_py("try:\n    x = 1\nexcept ValueError:\n    raise");
        assert!(issues.iter().any(|i| i.rule == "pointless-except"));
    }

    #[test]
    fn detects_python_pass_body_function() {
        let issues = lint_py("def foo():\n    pass");
        assert!(issues.iter().any(|i| i.rule == "no-pass-body"));
    }

    #[test]
    fn detects_python_pass_body_class() {
        let issues = lint_py("class Foo:\n    pass");
        assert!(issues.iter().any(|i| i.rule == "no-pass-body"));
    }

    #[test]
    fn no_pass_body_when_has_code() {
        let issues = lint_py("def foo():\n    return 1");
        assert!(!issues.iter().any(|i| i.rule == "no-pass-body"));
    }

    #[test]
    fn detects_python_star_import() {
        let issues = lint_py("from os import *");
        assert!(issues.iter().any(|i| i.rule == "no-star-import"));
    }

    #[test]
    fn no_star_import_for_specific() {
        let issues = lint_py("from os import path");
        assert!(!issues.iter().any(|i| i.rule == "no-star-import"));
    }

    #[test]
    fn detects_python_nested_with() {
        let issues = lint_py("with open('a') as f:\n    with open('b') as g:\n        pass");
        assert!(issues.iter().any(|i| i.rule == "no-nested-with"));
    }

    #[test]
    fn no_nested_with_when_multiple_stmts() {
        let issues = lint_py("with open('a') as f:\n    x = 1\n    with open('b') as g:\n        pass");
        assert!(!issues.iter().any(|i| i.rule == "no-nested-with"));
    }

    #[test]
    fn detects_python_return_in_init() {
        let issues = lint_py("class Foo:\n    def __init__(self):\n        return 1");
        assert!(issues.iter().any(|i| i.rule == "no-return-in-init"));
    }

    #[test]
    fn no_return_in_init_for_bare_return() {
        let issues = lint_py("class Foo:\n    def __init__(self):\n        return");
        assert!(!issues.iter().any(|i| i.rule == "no-return-in-init"));
    }

    #[test]
    fn detects_python_simplify_boolean_return() {
        let issues = lint_py("def f(x):\n    if x:\n        return True\n    else:\n        return False");
        assert!(issues.iter().any(|i| i.rule == "simplify-boolean-return"));
    }

    #[test]
    fn no_simplify_boolean_return_for_non_bool() {
        let issues = lint_py("def f(x):\n    if x:\n        return 1\n    else:\n        return 0");
        assert!(!issues.iter().any(|i| i.rule == "simplify-boolean-return"));
    }

    // --- Java ---
    #[test]
    fn detects_java_sysout() {
        let issues = lint_java("class T { void m() { System.out.println(\"hi\"); } }");
        assert!(issues.iter().any(|i| i.rule == "no-sysout"));
    }

    #[test]
    fn detects_java_empty_catch() {
        let issues = lint_java("class T { void m() { try {} catch (Exception e) {} } }");
        assert!(issues.iter().any(|i| i.rule == "no-empty-catch"));
    }

    // --- JS/TS (additional rules) ---
    #[test]
    fn detects_constant_condition() {
        let issues = lint_js("if (true) {}");
        assert!(issues.iter().any(|i| i.rule == "no-constant-condition"));
    }

    #[test]
    fn detects_self_compare() {
        let issues = lint_js("if (x === x) {}");
        assert!(issues.iter().any(|i| i.rule == "no-self-compare"));
    }

    #[test]
    fn detects_java_string_equality() {
        let issues = lint_java("class T { void m() { if (\"a\" == \"b\") {} } }");
        assert!(issues.iter().any(|i| i.rule == "no-string-equality"));
    }

    // --- C/C++ ---
    #[test]
    fn detects_c_gets() {
        let issues = lint_c("void f() { char buf[10]; gets(buf); }");
        assert!(issues.iter().any(|i| i.rule == "no-unsafe-fn"));
    }

    #[test]
    fn detects_c_goto() {
        let issues = lint_c("void f() { goto end; end: return; }");
        assert!(issues.iter().any(|i| i.rule == "no-goto"));
    }

    // --- JS/TS (bug detection) ---
    #[test]
    fn detects_useless_catch() {
        let issues = lint_js("try { foo(); } catch (e) { throw e; }");
        assert!(issues.iter().any(|i| i.rule == "no-useless-catch"));
    }

    #[test]
    fn no_issue_for_useful_catch() {
        let issues = lint_js("try { foo(); } catch (e) { console.log(e); throw e; }");
        assert!(!issues.iter().any(|i| i.rule == "no-useless-catch"));
    }

    #[test]
    fn detects_use_isnan() {
        let issues = lint_js("if (x === NaN) {}");
        assert!(issues.iter().any(|i| i.rule == "use-isnan"));
    }

    #[test]
    fn no_issue_for_number_isnan() {
        let issues = lint_js("if (Number.isNaN(x)) {}");
        assert!(!issues.iter().any(|i| i.rule == "use-isnan"));
    }

    #[test]
    fn detects_compare_neg_zero() {
        let issues = lint_js("if (x === -0) {}");
        assert!(issues.iter().any(|i| i.rule == "no-compare-neg-zero"));
    }

    #[test]
    fn no_issue_for_compare_zero() {
        let issues = lint_js("if (x === 0) {}");
        assert!(!issues.iter().any(|i| i.rule == "no-compare-neg-zero"));
    }

    #[test]
    fn detects_unsafe_negation_in() {
        let issues = lint_js("if (!key in obj) {}");
        assert!(issues.iter().any(|i| i.rule == "no-unsafe-negation"));
    }

    #[test]
    fn detects_unsafe_negation_instanceof() {
        let issues = lint_js("if (!x instanceof Cls) {}");
        assert!(issues.iter().any(|i| i.rule == "no-unsafe-negation"));
    }

    #[test]
    fn detects_constructor_return() {
        let issues = lint_js("class Foo { constructor() { return {}; } }");
        assert!(issues.iter().any(|i| i.rule == "no-constructor-return"));
    }

    #[test]
    fn no_issue_for_empty_constructor_return() {
        let issues = lint_js("class Foo { constructor() { return; } }");
        assert!(!issues.iter().any(|i| i.rule == "no-constructor-return"));
    }

    #[test]
    fn detects_invalid_typeof() {
        let issues = lint_js("if (typeof x === 'strig') {}");
        assert!(issues.iter().any(|i| i.rule == "valid-typeof"));
    }

    #[test]
    fn no_issue_for_valid_typeof() {
        let issues = lint_js("if (typeof x === 'string') {}");
        assert!(!issues.iter().any(|i| i.rule == "valid-typeof"));
    }

    // --- JS/TS (style) ---
    #[test]
    fn detects_useless_rename_import() {
        let issues = lint_js("import { foo as foo } from 'bar';");
        assert!(issues.iter().any(|i| i.rule == "no-useless-rename"));
    }

    #[test]
    fn no_issue_for_useful_rename_import() {
        let issues = lint_js("import { foo as bar } from 'baz';");
        assert!(!issues.iter().any(|i| i.rule == "no-useless-rename"));
    }

    #[test]
    fn detects_useless_rename_destructure() {
        let issues = lint_js("const { a: a } = obj;");
        assert!(issues.iter().any(|i| i.rule == "no-useless-rename"));
    }

    #[test]
    fn detects_empty_object_pattern() {
        let issues = lint_js("const {} = obj;");
        assert!(issues.iter().any(|i| i.rule == "no-empty-pattern"));
    }

    #[test]
    fn detects_empty_array_pattern() {
        let issues = lint_js("const [] = arr;");
        assert!(issues.iter().any(|i| i.rule == "no-empty-pattern"));
    }

    #[test]
    fn no_issue_for_nonempty_pattern() {
        let issues = lint_js("const { a } = obj;");
        assert!(!issues.iter().any(|i| i.rule == "no-empty-pattern"));
    }

    // --- Config ---
    #[test]
    fn disabled_rule_not_reported() {
        let json = r#"{ "rules": { "no-var": false } }"#;
        let config: CodopsyConfig = serde_json::from_str(json).unwrap();
        let tree = parse_source("var x = 1;", SourceLanguage::JavaScript).unwrap();
        let issues = lint_file("test.js", "var x = 1;", &tree, &config, SourceLanguage::JavaScript);
        assert!(!issues.iter().any(|i| i.rule == "no-var"));
    }

    // --- Universal ---
    #[test]
    fn todo_comment_detected_for_all_languages() {
        let issues = lint_py("# TODO: fix this");
        assert!(issues.iter().any(|i| i.rule == "todo-comment"));
    }

    #[test]
    fn universal_empty_function_for_python() {
        let issues = lint_py("def foo():\n    pass");
        assert!(!issues.iter().any(|i| i.rule == "no-empty-function"));
    }

    // --- Rust (Clippy-inspired) ---
    #[test]
    fn detects_needless_return() {
        let issues = lint_rs("fn foo() -> i32 { return 42; }");
        assert!(issues.iter().any(|i| i.rule == "needless-return"));
    }

    #[test]
    fn no_needless_return_for_early_return() {
        let issues = lint_rs("fn foo(x: bool) -> i32 { if x { return 1; } 2 }");
        assert!(!issues.iter().any(|i| i.rule == "needless-return"));
    }

    #[test]
    fn detects_bool_comparison_eq_true() {
        let issues = lint_rs("fn foo(x: bool) { if x == true {} }");
        assert!(issues.iter().any(|i| i.rule == "bool-comparison"));
    }

    #[test]
    fn detects_bool_comparison_ne_false() {
        let issues = lint_rs("fn foo(x: bool) { if x != false {} }");
        assert!(issues.iter().any(|i| i.rule == "bool-comparison"));
    }

    #[test]
    fn no_bool_comparison_for_normal_eq() {
        let issues = lint_rs("fn foo(x: i32) { if x == 1 {} }");
        assert!(!issues.iter().any(|i| i.rule == "bool-comparison"));
    }

    #[test]
    fn detects_collapsible_if() {
        let issues = lint_rs("fn foo(a: bool, b: bool) { if a { if b { println!(\"x\"); } } }");
        assert!(issues.iter().any(|i| i.rule == "collapsible-if"));
    }

    #[test]
    fn no_collapsible_if_with_else() {
        let issues = lint_rs("fn foo(a: bool, b: bool) { if a { if b { } } else { } }");
        assert!(!issues.iter().any(|i| i.rule == "collapsible-if"));
    }

    #[test]
    fn detects_single_match() {
        let issues = lint_rs("fn foo(x: Option<i32>) { match x { Some(v) => println!(\"{}\", v), _ => () } }");
        assert!(issues.iter().any(|i| i.rule == "single-match"));
    }

    #[test]
    fn detects_manual_map() {
        let issues = lint_rs("fn foo(x: Option<i32>) -> Option<i32> { match x { Some(v) => Some(v + 1), None => None } }");
        assert!(issues.iter().any(|i| i.rule == "manual-map"));
    }

    #[test]
    fn detects_redundant_clone() {
        let issues = lint_rs("fn foo(s: String) { let _ = s.clone().clone(); }");
        assert!(issues.iter().any(|i| i.rule == "redundant-clone"));
    }

    #[test]
    fn no_redundant_clone_single() {
        let issues = lint_rs("fn foo(s: String) { let _ = s.clone(); }");
        assert!(!issues.iter().any(|i| i.rule == "redundant-clone"));
    }

    #[test]
    fn detects_eq_op() {
        let issues = lint_rs("fn foo(x: i32) { if x == x {} }");
        assert!(issues.iter().any(|i| i.rule == "eq-op"));
    }

    #[test]
    fn no_eq_op_different_sides() {
        let issues = lint_rs("fn foo(x: i32, y: i32) { if x == y {} }");
        assert!(!issues.iter().any(|i| i.rule == "eq-op"));
    }
}
