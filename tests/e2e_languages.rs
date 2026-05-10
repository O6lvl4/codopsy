//! End-to-end tests: run codopsy analysis on fixture files and verify
//! that the expected lint rules fire for each language.

use std::collections::HashSet;
use std::path::PathBuf;

use codopsy::analyzer::analyze_file;
use codopsy::config::CodopsyConfig;

/// Parse `// expect: rule-a, rule-b` or `# expect: rule-a, rule-b` comments
/// from the first lines of a fixture file.
fn parse_expected_rules(source: &str) -> HashSet<String> {
    let mut rules = HashSet::new();
    for line in source.lines() {
        let trimmed = line.trim();
        // Match both // and # comment styles
        let after = if let Some(rest) = trimmed.strip_prefix("//") {
            rest.trim()
        } else if let Some(rest) = trimmed.strip_prefix('#') {
            rest.trim()
        } else if let Some(rest) = trimmed.strip_prefix(';') {
            rest.trim()
        } else {
            continue;
        };
        if let Some(rest) = after.strip_prefix("expect:") {
            for rule in rest.split(',') {
                let r = rule.trim();
                if !r.is_empty() {
                    rules.insert(r.to_string());
                }
            }
        }
    }
    rules
}

fn fixture_path(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path.to_string_lossy().to_string()
}

fn run_fixture(filename: &str) {
    let path = fixture_path(filename);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {filename}: {e}"));
    let expected = parse_expected_rules(&source);
    assert!(
        !expected.is_empty(),
        "Fixture {filename} has no `expect:` comments"
    );

    let config = CodopsyConfig::default();
    let analysis = analyze_file(&path, &config);
    let fired: HashSet<String> = analysis.issues.iter().map(|i| i.rule.clone()).collect();

    let mut missing: Vec<&String> = expected.difference(&fired).collect();
    missing.sort();

    assert!(
        missing.is_empty(),
        "Fixture {filename}: expected rules not fired: {:?}\n  fired: {:?}",
        missing,
        fired
    );
}

#[test]
fn e2e_javascript() {
    run_fixture("js_violations.js");
}

#[test]
fn e2e_typescript() {
    run_fixture("ts_violations.ts");
}

#[test]
fn e2e_rust() {
    run_fixture("rust_violations.rs");
}

#[test]
fn e2e_go() {
    run_fixture("go_violations.go");
}

#[test]
fn e2e_python() {
    run_fixture("python_violations.py");
}

#[test]
fn e2e_java() {
    run_fixture("java_violations.java");
}

#[test]
fn e2e_c() {
    run_fixture("c_violations.c");
}

#[test]
fn e2e_elixir() {
    run_fixture("elixir_violations.ex");
}

#[test]
fn e2e_clojure() {
    run_fixture("clojure_violations.clj");
}

#[test]
fn e2e_gleam() {
    run_fixture("gleam_violations.gleam");
}

/// Verify that a clean file produces no issues (except threshold rules).
#[test]
fn e2e_clean_file_no_violations() {
    let source = "function add(a, b) { return a + b; }\n";
    let tmp = std::env::temp_dir().join("codopsy_e2e_clean.js");
    std::fs::write(&tmp, source).unwrap();

    let config = CodopsyConfig::default();
    let analysis = analyze_file(&tmp.to_string_lossy(), &config);

    let lint_issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| !i.rule.starts_with("max-"))
        .collect();
    assert!(
        lint_issues.is_empty(),
        "Clean file should have no lint issues, got: {:?}",
        lint_issues.iter().map(|i| &i.rule).collect::<Vec<_>>()
    );

    let _ = std::fs::remove_file(&tmp);
}
