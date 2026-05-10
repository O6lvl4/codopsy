//! Linter comparison E2E tests.
//!
//! Runs codopsy alongside each language's native linter on the same file,
//! then prints a coverage comparison report. Tests always pass — they report,
//! not assert.
//!
//! Run: `cargo test --test e2e_compare -- --nocapture`
//!
//! Native linters are auto-detected. When unavailable, codopsy-only results
//! are shown. Install linters via qusp or system package manager to enable
//! full comparison.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use codopsy::analyzer::analyze_file;
use codopsy::config::CodopsyConfig;

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/compare");
    p.push(name);
    p
}

fn run_codopsy(path: &PathBuf) -> BTreeSet<String> {
    let config = CodopsyConfig::default();
    let analysis = analyze_file(&path.to_string_lossy(), &config);
    analysis
        .issues
        .iter()
        .map(|i| format!("L{}:{} [{}] {}", i.line, i.column, i.rule, i.message))
        .collect()
}

fn try_run(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", augmented_path())
        .output()
        .ok()?;
    Some(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// Augment PATH to include common linter install locations.
fn augmented_path() -> String {
    let base = std::env::var("PATH").unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_default();
    let extras = [
        format!("{home}/.local/bin"),
        format!("{home}/.cargo/bin"),
        "/usr/local/bin".to_string(),
        "/opt/homebrew/bin".to_string(),
    ];
    format!("{}:{}", extras.join(":"), base)
}

fn try_qusp(args: &[&str]) -> Option<String> {
    let full: Vec<&str> = [&["run"], args].concat();
    try_run("qusp", &full)
}

/// Try multiple commands in order, return the first that succeeds.
fn try_any(attempts: &[(&str, &[&str])]) -> (Option<String>, &'static str) {
    for &(name, args_slice) in attempts {
        // Split the first element as command, rest as args
        // Actually we receive (label, full_args_to_try_run)
        // Let's restructure
    }
    (None, "none")
}

fn print_report(lang: &str, file: &str, codopsy_issues: &BTreeSet<String>, native: Option<&str>, native_name: &str) {
    let native_count = native.map(|o| {
        o.lines().filter(|l| !l.trim().is_empty()).count()
    }).unwrap_or(0);

    eprintln!();
    eprintln!("┌─────────────────────────────────────────────────────────────────┐");
    eprintln!("│ {:10} │ codopsy: {:3} issues │ {:>15}: ~{:<3} lines │",
        lang, codopsy_issues.len(), native_name, native_count);
    eprintln!("├─────────────────────────────────────────────────────────────────┤");

    for issue in codopsy_issues {
        eprintln!("│ codopsy │ {}", issue);
    }

    if let Some(output) = native {
        eprintln!("│─────────┤");
        for line in output.lines().filter(|l| !l.trim().is_empty()).take(20) {
            eprintln!("│ {:7} │ {}", native_name, line.trim());
        }
        let total = output.lines().filter(|l| !l.trim().is_empty()).count();
        if total > 20 {
            eprintln!("│ {:7} │ ... +{} more lines", native_name, total - 20);
        }
    } else {
        eprintln!("│ {:7} │ (not installed — skipped)", native_name);
    }

    eprintln!("└─────────────────────────────────────────────────────────────────┘");
}

fn compare(lang: &str, file: &str, native_name: &str, native_fn: impl FnOnce(&str) -> Option<String>) {
    let path = fixture(file);
    let codopsy = run_codopsy(&path);
    let path_str = path.to_string_lossy().to_string();
    let native = native_fn(&path_str);
    print_report(lang, file, &codopsy, native.as_deref(), native_name);
}

// ─── Languages with dedicated codopsy rules ──────────────────────────

#[test]
fn compare_javascript() {
    compare("JavaScript", "sample.js", "ESLint", |f| {
        try_qusp(&[
            "npx", "--yes", "eslint@latest", "--no-eslintrc",
            "--rule", r#"{"no-var":"warn","eqeqeq":"warn","no-eval":"error","no-debugger":"error","no-unreachable":"error","valid-typeof":"error","use-isnan":"error","no-console":"warn","no-constant-condition":"warn"}"#,
            f,
        ])
    });
}

#[test]
fn compare_typescript() {
    compare("TypeScript", "sample.ts", "ESLint", |f| {
        try_qusp(&[
            "npx", "--yes", "eslint@latest", "--no-eslintrc",
            "--rule", r#"{"no-var":"warn","no-eval":"error","no-debugger":"error","no-unreachable":"error","use-isnan":"error","no-console":"warn"}"#,
            f,
        ])
    });
}

#[test]
fn compare_rust() {
    compare("Rust", "sample.rs", "Clippy", |f| {
        try_run("clippy-driver", &["--edition", "2021", "-W", "clippy::all", f])
            .or_else(|| try_run("rustc", &["--edition", "2021", "--crate-type", "lib", "-W", "warnings", f, "-o", "/dev/null"]))
    });
}

#[test]
fn compare_go() {
    compare("Go", "sample.go", "go vet", |f| {
        try_qusp(&["go", "vet", f])
    });
}

#[test]
fn compare_python() {
    compare("Python", "sample.py", "Ruff", |f| {
        try_qusp(&["ruff", "check", "--select", "ALL", "--no-fix", f])
            .or_else(|| try_run("ruff", &["check", "--select", "ALL", "--no-fix", f]))
    });
}

#[test]
fn compare_java() {
    compare("Java", "sample.java", "javac", |f| {
        try_qusp(&["javac", "-Xlint:all", "-d", "/tmp", f])
            .or_else(|| try_run("javac", &["-Xlint:all", "-d", "/tmp", f]))
    });
}

#[test]
fn compare_c() {
    compare("C", "sample.c", "gcc", |f| {
        try_run("gcc", &["-Wall", "-Wextra", "-fsyntax-only", f])
            .or_else(|| try_run("clang", &["-Wall", "-Wextra", "-fsyntax-only", f]))
    });
}

#[test]
fn compare_elixir() {
    compare("Elixir", "sample.ex", "Credo", |f| {
        try_qusp(&["mix", "credo", "--strict", f])
    });
}

#[test]
fn compare_erlang() {
    compare("Erlang", "sample.erl", "erlc", |f| {
        try_run("erlc", &["-W", "+warn_unused_vars", f])
    });
}

#[test]
fn compare_clojure() {
    compare("Clojure", "sample.clj", "clj-kondo", |f| {
        try_run("clj-kondo", &["--lint", f])
            .or_else(|| try_qusp(&["clj-kondo", "--lint", f]))
    });
}

#[test]
fn compare_gleam() {
    compare("Gleam", "sample.gleam", "gleam", |f| {
        // gleam doesn't lint single files easily, just report codopsy
        let _ = f;
        None
    });
}

// ─── Languages with universal rules only ─────────────────────────────

#[test]
fn compare_ruby() {
    compare("Ruby", "sample.rb", "RuboCop", |f| {
        try_run("rubocop", &["--format", "simple", f])
            .or_else(|| try_qusp(&["rubocop", "--format", "simple", f]))
    });
}

#[test]
fn compare_swift() {
    compare("Swift", "sample.swift", "swiftc", |f| {
        try_run("swiftc", &["-typecheck", f])
    });
}

#[test]
fn compare_lua() {
    compare("Lua", "sample.lua", "luacheck", |f| {
        try_run("luacheck", &["--no-color", f])
            .or_else(|| try_qusp(&["luacheck", "--no-color", f]))
    });
}

#[test]
fn compare_bash() {
    compare("Bash", "sample.sh", "ShellCheck", |f| {
        try_run("shellcheck", &["-f", "gcc", f])
    });
}

#[test]
fn compare_zig() {
    compare("Zig", "sample.zig", "zig", |f| {
        try_run("zig", &["ast-check", f])
    });
}

#[test]
fn compare_haskell() {
    compare("Haskell", "sample.hs", "HLint", |f| {
        try_run("hlint", &[f])
    });
}

#[test]
fn compare_scala() {
    compare("Scala", "sample.scala", "scalac", |f| {
        try_qusp(&["scalac", "-deprecation", "-feature", f])
    });
}

#[test]
fn compare_php() {
    compare("PHP", "sample.php", "php", |f| {
        try_run("php", &["-l", f])
            .or_else(|| try_qusp(&["php", "-l", f]))
    });
}

#[test]
fn compare_csharp() {
    compare("C#", "sample.cs", "dotnet", |_f| {
        // dotnet requires project setup, skip
        None
    });
}
