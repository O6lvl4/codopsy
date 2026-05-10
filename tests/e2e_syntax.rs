//! Syntax validation E2E tests.
//!
//! Uses `qusp run` to invoke each language's native toolchain to verify that
//! "clean" fixture files are syntactically valid. This ensures codopsy's
//! tree-sitter parsing agrees with the real compiler/interpreter.
//!
//! Run: `cargo test --test e2e_syntax`
//! Requires: `qusp install` (installs toolchains from qusp.toml)

use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    path
}

fn has_qusp() -> bool {
    Command::new("qusp")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run a syntax check command via `qusp run` and assert it succeeds.
fn qusp_syntax_check(args: &[&str], fixture: &str) {
    if !has_qusp() {
        eprintln!("  [skip] qusp not available");
        return;
    }

    let file = fixture_path(fixture);
    let full_args: Vec<String> = args
        .iter()
        .map(|a| {
            if *a == "{file}" {
                file.to_string_lossy().to_string()
            } else {
                a.to_string()
            }
        })
        .collect();

    let output = Command::new("qusp")
        .arg("run")
        .args(&full_args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();

    match output {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let stdout = String::from_utf8_lossy(&o.stdout);
                panic!(
                    "Syntax check failed for {fixture}:\n  cmd: qusp run {}\n  stderr: {stderr}\n  stdout: {stdout}",
                    full_args.join(" ")
                );
            }
        }
        Err(e) => {
            eprintln!("  [skip] qusp run failed: {e}");
        }
    }
}

/// Also verify that codopsy produces zero lint issues on clean files.
fn codopsy_clean_check(fixture: &str) {
    let path = fixture_path(fixture).to_string_lossy().to_string();
    let config = codopsy::config::CodopsyConfig::default();
    let analysis = codopsy::analyzer::analyze_file(&path, &config);
    let lint_issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| !i.rule.starts_with("max-"))
        .collect();
    assert!(
        lint_issues.is_empty(),
        "Clean fixture {fixture} should have no lint issues, got: {:?}",
        lint_issues.iter().map(|i| format!("{}: {}", i.rule, i.message)).collect::<Vec<_>>()
    );
}

#[test]
fn syntax_go_clean() {
    qusp_syntax_check(&["go", "vet", "{file}"], "go_clean.go");
    codopsy_clean_check("go_clean.go");
}

#[test]
fn syntax_python_clean() {
    qusp_syntax_check(&["python", "-m", "py_compile", "{file}"], "python_clean.py");
    codopsy_clean_check("python_clean.py");
}

#[test]
fn syntax_rust_clean() {
    // rustc --edition 2021 parse check only
    let tmp_out = std::env::temp_dir().join("codopsy_e2e_rust_clean.rlib");
    let tmp_str = tmp_out.to_string_lossy().to_string();
    qusp_syntax_check(
        &["rustc", "--edition", "2021", "--crate-type", "lib", "-A", "warnings", "{file}", "-o", &tmp_str],
        "rust_clean.rs",
    );
    let _ = std::fs::remove_file(&tmp_out);
    codopsy_clean_check("rust_clean.rs");
}

#[test]
fn syntax_js_node_clean() {
    // node --check parses without executing
    qusp_syntax_check(&["node", "--check", "{file}"], "js_violations.js");
    // (we don't assert clean here — violations file is intentionally bad for lint)
}
