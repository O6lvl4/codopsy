//! The rule set for each language, as data.
//!
//! Which rules exist per language lives here; how a language selects its set
//! lives in `rule_registry`.

use crate::types::Severity;
use super::rules::bash_rules;
use super::rules::bug_detection::*;
use super::rules::c_rules::*;
use super::rules::clojure_rules;
use super::rules::control_flow::*;
use super::rules::crystal_rules;
use super::rules::dart_rules;
use super::rules::elixir_rules::*;
use super::rules::elm_rules;
use super::rules::erlang_rules::*;
use super::rules::gleam_rules;
use super::rules::go_extra_rules;
use super::rules::go_rules;
use super::rules::groovy_rules;
use super::rules::haskell_rules;
use super::rules::java_rules::*;
use super::rules::julia_rules;
use super::rules::kotlin_rules;
use super::rules::lean_rules;
use super::rules::lua_rules;
use super::rules::php_rules;
use super::rules::python_flow_rules::*;
use super::rules::python_rules::*;
use super::rules::ruby_rules;
use super::rules::rust_extra_rules;
use super::rules::rust_rules;
use super::rules::scala_rules;
use super::rules::style_rules::*;
use super::rules::swift_rules;
use super::rules::zig_rules;
use super::rule_registry::SimpleCheckFn;

pub const JS_TS_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
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
pub const RUST_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
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
    ("collapsible-if", Severity::Warning, rust_extra_rules::check_collapsible_if),
    ("single-match", Severity::Warning, rust_extra_rules::check_single_match),
    ("manual-map", Severity::Warning, rust_extra_rules::check_manual_map),
    ("redundant-clone", Severity::Warning, rust_extra_rules::check_redundant_clone),
    ("eq-op", Severity::Warning, rust_extra_rules::check_eq_op),
];
pub const GO_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-panic", Severity::Warning, go_rules::check_no_panic),
    ("no-fmt-print", Severity::Info, go_rules::check_no_fmt_print),
    ("no-ignored-error", Severity::Warning, go_rules::check_no_ignored_error),
    ("no-os-exit", Severity::Warning, go_rules::check_no_os_exit),
    ("no-defer-in-loop", Severity::Warning, go_rules::check_no_defer_in_loop),
    ("no-empty-block", Severity::Warning, go_rules::check_no_empty_block),
    ("no-unreachable", Severity::Error, go_rules::check_no_unreachable_go),
    ("no-naked-return", Severity::Warning, go_rules::check_no_naked_return),
    ("no-range-over-string", Severity::Info, go_rules::check_no_range_over_string),
    ("no-shadow-import", Severity::Warning, go_extra_rules::check_no_shadow_import),
    ("collapsible-if", Severity::Warning, go_extra_rules::check_collapsible_if_go),
    ("superfluous-else", Severity::Warning, go_extra_rules::check_superfluous_else_go),
];
pub const PYTHON_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
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
    ("collapsible-if", Severity::Warning, check_collapsible_if_python),
    ("superfluous-else", Severity::Warning, check_superfluous_else),
];
pub const JAVA_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-sysout", Severity::Warning, check_no_sysout),
    ("no-print-stack-trace", Severity::Warning, check_no_print_stack_trace),
    ("no-empty-catch", Severity::Warning, check_no_empty_catch),
    ("no-throws-exception", Severity::Warning, check_no_throws_exception),
    ("no-raw-type", Severity::Warning, check_no_raw_type),
    ("no-string-equality", Severity::Warning, check_no_string_equality),
    ("missing-switch-default", Severity::Warning, check_no_missing_switch_default),
    ("no-empty-if", Severity::Warning, check_no_empty_if_java),
    ("no-double-brace-init", Severity::Warning, check_no_double_brace_init),
    ("no-string-concat-in-loop", Severity::Warning, check_no_string_concat_in_loop),
    ("no-nested-try", Severity::Warning, check_no_nested_try),
    ("equals-null", Severity::Error, check_equals_null),
];
pub const C_CPP_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-printf", Severity::Info, check_no_printf),
    ("no-unsafe-fn", Severity::Error, check_no_unsafe_fn),
    ("no-malloc", Severity::Info, check_no_malloc),
    ("no-goto", Severity::Warning, check_no_goto),
    ("no-sizeof-ptr", Severity::Warning, check_no_sizeof_ptr),
    ("no-magic-number", Severity::Info, check_no_magic_number),
    ("no-implicit-fallthrough", Severity::Warning, check_no_implicit_fallthrough_c),
    ("no-empty-if", Severity::Warning, check_no_empty_if_c),
    ("no-void-main", Severity::Warning, check_no_void_main),
];
pub const ELIXIR_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-io-inspect", Severity::Warning, check_no_io_inspect),
    ("no-io-puts", Severity::Info, check_no_io_puts),
    ("no-raise-in-with", Severity::Warning, check_no_raise_in_with),
    ("pipe-into-anonymous", Severity::Warning, check_pipe_into_anonymous),
];
pub const ERLANG_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-process-flag", Severity::Warning, check_no_process_flag),
    ("no-catch-all", Severity::Warning, check_no_catch_all),
    ("no-exit-call", Severity::Warning, check_no_exit_call),
];
pub const GLEAM_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-todo", Severity::Warning, gleam_rules::check_no_todo),
    ("no-panic", Severity::Warning, gleam_rules::check_no_panic),
    ("no-let-assert", Severity::Warning, gleam_rules::check_no_let_assert),
];
pub const CLOJURE_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, clojure_rules::check_no_println),
    ("no-def-in-def", Severity::Warning, clojure_rules::check_no_def_in_def),
    ("no-thread-sleep", Severity::Warning, clojure_rules::check_no_thread_sleep),
    ("no-reflection", Severity::Warning, clojure_rules::check_no_reflection),
];
pub const RUBY_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-puts", Severity::Info, ruby_rules::check_no_puts),
    ("no-eval", Severity::Error, ruby_rules::check_no_eval),
    ("require-relative", Severity::Warning, ruby_rules::check_require_relative),
    ("no-rescue-exception", Severity::Warning, ruby_rules::check_no_rescue_exception),
    ("no-sleep", Severity::Warning, ruby_rules::check_no_sleep),
];
pub const PHP_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-debug-output", Severity::Warning, php_rules::check_no_debug_output),
    ("no-eval", Severity::Error, php_rules::check_no_eval),
    ("no-exit", Severity::Warning, php_rules::check_no_exit),
    ("strict-comparison", Severity::Warning, php_rules::check_strict_comparison),
    ("no-error-suppression", Severity::Warning, php_rules::check_no_error_suppression),
];
pub const LUA_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-global", Severity::Warning, lua_rules::check_no_global),
    ("no-os-execute", Severity::Error, lua_rules::check_no_os_execute),
    ("no-loadstring", Severity::Error, lua_rules::check_no_loadstring),
    ("no-print", Severity::Info, lua_rules::check_no_print),
];
pub const SWIFT_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-print", Severity::Info, swift_rules::check_no_print),
    ("no-force-unwrap", Severity::Warning, swift_rules::check_no_force_unwrap),
    ("no-force-try", Severity::Warning, swift_rules::check_no_force_try),
    ("no-force-cast", Severity::Warning, swift_rules::check_no_force_cast),
    ("no-nslog", Severity::Warning, swift_rules::check_no_nslog),
    ("no-fatal-error", Severity::Warning, swift_rules::check_no_fatal_error),
];
pub const ZIG_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-debug-print", Severity::Info, zig_rules::check_no_debug_print),
    ("no-unreachable", Severity::Warning, zig_rules::check_no_unreachable),
    ("no-panic", Severity::Warning, zig_rules::check_no_panic),
    ("no-catch-all-switch", Severity::Warning, zig_rules::check_no_catch_all_switch),
];
pub const HASKELL_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-partial-function", Severity::Warning, haskell_rules::check_no_partial_functions),
    ("no-undefined", Severity::Warning, haskell_rules::check_no_undefined),
    ("no-error", Severity::Warning, haskell_rules::check_no_error),
    ("no-unsafe-perform-io", Severity::Error, haskell_rules::check_no_unsafe_perform_io),
    ("no-trace", Severity::Warning, haskell_rules::check_no_trace),
];
pub const SCALA_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, scala_rules::check_no_println),
    ("no-null", Severity::Warning, scala_rules::check_no_null),
    ("no-var", Severity::Warning, scala_rules::check_no_var),
    ("no-return", Severity::Warning, scala_rules::check_no_return),
    ("no-as-instance-of", Severity::Warning, scala_rules::check_no_as_instance_of),
];
pub const KOTLIN_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, kotlin_rules::check_no_println),
    ("no-unsafe-cast", Severity::Warning, kotlin_rules::check_no_unsafe_cast),
    ("no-not-null-assertion", Severity::Warning, kotlin_rules::check_no_not_null_assertion),
    ("no-empty-catch", Severity::Warning, kotlin_rules::check_no_empty_catch),
    ("no-system-exit", Severity::Warning, kotlin_rules::check_no_system_exit),
    ("prefer-val", Severity::Info, kotlin_rules::check_prefer_val),
];
pub const CRYSTAL_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-puts", Severity::Info, crystal_rules::check_no_puts),
    ("no-raise-string", Severity::Warning, crystal_rules::check_no_raise_string),
    ("no-rescue-exception", Severity::Warning, crystal_rules::check_no_rescue_exception),
    ("no-shell", Severity::Error, crystal_rules::check_no_shell),
    ("no-sleep", Severity::Warning, crystal_rules::check_no_sleep),
];
pub const DART_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-print", Severity::Info, dart_rules::check_no_print),
    ("no-dynamic", Severity::Warning, dart_rules::check_no_dynamic),
    ("no-empty-catch", Severity::Warning, dart_rules::check_no_empty_catch),
    ("no-cast", Severity::Warning, dart_rules::check_no_cast),
    ("no-rethrow-only", Severity::Warning, dart_rules::check_no_rethrow_only),
];
pub const ELM_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-debug", Severity::Warning, elm_rules::check_no_debug),
    ("no-todo", Severity::Warning, elm_rules::check_no_todo),
    ("unused-import", Severity::Warning, elm_rules::check_unused_import),
];
pub const GROOVY_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, groovy_rules::check_no_println),
    ("no-def-type", Severity::Warning, groovy_rules::check_no_def_type),
    ("no-system-exit", Severity::Warning, groovy_rules::check_no_system_exit),
    ("no-empty-catch", Severity::Warning, groovy_rules::check_no_empty_catch),
];
pub const JULIA_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-println", Severity::Info, julia_rules::check_no_println),
    ("no-eval", Severity::Error, julia_rules::check_no_eval),
    ("no-global-mutable", Severity::Warning, julia_rules::check_no_global_mutable),
    ("no-bare-ccall", Severity::Warning, julia_rules::check_no_bare_ccall),
];
pub const LEAN_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("no-sorry", Severity::Error, lean_rules::check_no_sorry),
    ("no-axiom", Severity::Warning, lean_rules::check_no_axiom),
    ("no-native-decide", Severity::Warning, lean_rules::check_no_native_decide),
    ("no-unsafe", Severity::Warning, lean_rules::check_no_unsafe),
    ("no-unlimited-heartbeats", Severity::Warning, lean_rules::check_no_unlimited_heartbeats),
    ("no-partial-def", Severity::Info, lean_rules::check_no_partial_def),
    ("no-dbg-trace", Severity::Info, lean_rules::check_no_dbg_trace),
    ("no-debug-command", Severity::Info, lean_rules::check_no_debug_command),
];
pub const BASH_RULES: &[(&str, Severity, SimpleCheckFn)] = &[
    ("unquoted-expansion", Severity::Warning, bash_rules::check_unquoted_expansion),
    ("no-eval", Severity::Error, bash_rules::check_no_eval),
    ("cd-without-or", Severity::Warning, bash_rules::check_cd_without_or),
    ("useless-cat", Severity::Warning, bash_rules::check_useless_cat),
    ("dangerous-rm", Severity::Error, bash_rules::check_dangerous_rm),
    ("no-set-e", Severity::Info, bash_rules::check_no_set_e),
    ("test-equals", Severity::Warning, bash_rules::check_test_equals),
];
