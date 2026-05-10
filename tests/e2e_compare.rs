//! Linter comparison E2E tests.
//!
//! Runs codopsy alongside each language's native linter on the same file,
//! then prints a coverage comparison report. Tests always pass — they report,
//! not assert.
//!
//! Run: `cargo test --test e2e_compare -- --nocapture`
//!
//! Supported native linters (auto-detected):
//!   JS:     npx eslint (via qusp run)
//!   Python: ruff / qusp run ruff
//!   Rust:   cargo clippy
//!   Go:     go vet + staticcheck / golangci-lint

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

fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Some(combined)
}

fn run_qusp(args: &[&str]) -> Option<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("qusp")
        .arg("run")
        .args(args)
        .current_dir(manifest_dir)
        .output()
        .ok()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Some(combined)
}

fn count_diagnostic_lines(output: &str) -> usize {
    output
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with("warning:")
                && !t.starts_with("Checking")
                && !t.starts_with("Finished")
                && !t.starts_with("error: could not compile")
                && !t.starts_with("For more information")
        })
        .count()
}

fn print_report(lang: &str, file: &str, codopsy_issues: &BTreeSet<String>, native_output: Option<&str>, native_name: &str) {
    eprintln!();
    eprintln!("╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║  {} — {} vs codopsy", lang, native_name);
    eprintln!("╠══════════════════════════════════════════════════════════════╣");
    eprintln!("║  File: {}", file);
    eprintln!("║  codopsy issues: {}", codopsy_issues.len());

    if let Some(output) = native_output {
        let native_lines = count_diagnostic_lines(output);
        eprintln!("║  {} diagnostics: ~{}", native_name, native_lines);
        eprintln!("╠══════════════════════════════════════════════════════════════╣");
        eprintln!("║  codopsy findings:");
        for issue in codopsy_issues {
            eprintln!("║    {}", issue);
        }
        eprintln!("║");
        eprintln!("║  {} output (raw):", native_name);
        for line in output.lines().take(30) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                eprintln!("║    {}", trimmed);
            }
        }
        if output.lines().count() > 30 {
            eprintln!("║    ... ({} more lines)", output.lines().count() - 30);
        }
    } else {
        eprintln!("║  {} not available — skipping comparison", native_name);
        eprintln!("╠══════════════════════════════════════════════════════════════╣");
        eprintln!("║  codopsy findings:");
        for issue in codopsy_issues {
            eprintln!("║    {}", issue);
        }
    }
    eprintln!("╚══════════════════════════════════════════════════════════════╝");
}

#[test]
fn compare_js_eslint() {
    let path = fixture("sample.js");
    let codopsy = run_codopsy(&path);

    // Try eslint via npx (no config = eslint recommended)
    let native = run_qusp(&[
        "npx", "--yes", "eslint@latest", "--no-eslintrc",
        "--rule", "{\"no-var\": \"warn\", \"eqeqeq\": \"warn\", \"no-eval\": \"error\", \"no-debugger\": \"error\", \"no-unreachable\": \"error\", \"valid-typeof\": \"error\", \"use-isnan\": \"error\"}",
        &path.to_string_lossy(),
    ]);

    print_report("JavaScript", "sample.js", &codopsy, native.as_deref(), "ESLint");
}

#[test]
fn compare_python_ruff() {
    let path = fixture("sample.py");
    let codopsy = run_codopsy(&path);

    // Try ruff (fast Python linter)
    let native = run_qusp(&[
        "ruff", "check", "--select", "ALL", "--no-fix",
        &path.to_string_lossy(),
    ]).or_else(|| {
        // Fallback: try pip-installed ruff
        run_command("ruff", &["check", "--select", "ALL", "--no-fix", &path.to_string_lossy()])
    });

    print_report("Python", "sample.py", &codopsy, native.as_deref(), "Ruff");
}

#[test]
fn compare_rust_clippy() {
    let path = fixture("sample.rs");
    let codopsy = run_codopsy(&path);

    // clippy requires a cargo project, so we use rustc with clippy driver
    // or just run clippy on the file directly
    let native = run_command(
        "clippy-driver",
        &["--edition", "2021", "-W", "clippy::all", &path.to_string_lossy()],
    ).or_else(|| {
        // Fallback: parse with rustc warnings
        run_command("rustc", &[
            "--edition", "2021", "--crate-type", "lib",
            "-W", "warnings",
            &path.to_string_lossy(),
            "-o", "/dev/null",
        ])
    });

    print_report("Rust", "sample.rs", &codopsy, native.as_deref(), "Clippy");
}

#[test]
fn compare_go_vet() {
    let path = fixture("sample.go");
    let codopsy = run_codopsy(&path);

    let native = run_qusp(&["go", "vet", &path.to_string_lossy()]);

    print_report("Go", "sample.go", &codopsy, native.as_deref(), "go vet");
}
