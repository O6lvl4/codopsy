//! Language-version awareness.
//!
//! Some syntax is only legal from a certain language version on. A grammar that
//! can *read* it (see the Go 1.27 generic-method support) still can't tell you
//! whether the project is *allowed* to use it — that depends on the version the
//! project declares in its manifest. This module reads that declared version and
//! flags features used below it, the way `go vet`'s `stdversion` check does.
//!
//! Today this covers one feature (Go generic methods, which require go1.27), but
//! the shape — resolve a declared version from a manifest, then gate features on
//! it — is meant to extend to other languages and features.

use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{node_column, node_line, SourceLanguage};
use crate::config::CodopsyConfig;
use crate::types::{Issue, Severity};

/// A `(major, minor)` language version, e.g. `(1, 27)`.
pub type Version = (u32, u32);

/// The go.mod version at or above which generic methods are legal.
const GO_GENERIC_METHODS_SINCE: Version = (1, 27);

/// Append version-gate issues for `tree` to `issues`.
///
/// Resolves the project's declared version from its manifest and reports syntax
/// that the declared version does not allow. A no-op for languages with no gate
/// implemented, and when no manifest/version can be found (we never guess).
pub fn check_version_features(
    tree: &Tree,
    language: SourceLanguage,
    file_path: &str,
    config: &CodopsyConfig,
    issues: &mut Vec<Issue>,
) {
    if language != SourceLanguage::Go {
        return;
    }
    check_go(tree, file_path, config, issues);
}

fn check_go(tree: &Tree, file_path: &str, config: &CodopsyConfig, issues: &mut Vec<Issue>) {
    const RULE: &str = "go-generic-method";
    if config.is_rule_disabled(RULE) {
        return;
    }
    // Only a version older than the feature's floor can be violated. No go.mod,
    // or a new-enough one, means nothing to flag.
    let Some(declared) = resolve_go_version(Path::new(file_path)) else {
        return;
    };
    if declared >= GO_GENERIC_METHODS_SINCE {
        return;
    }

    let severity = config.get_rule_severity(RULE).unwrap_or(Severity::Warning);
    let mut methods = Vec::new();
    collect_generic_methods(&tree.root_node(), &mut methods);
    for node in methods {
        issues.push(Issue {
            file: file_path.to_string(),
            line: node_line(&node),
            column: node_column(&node),
            severity,
            rule: RULE.to_string(),
            message: format!(
                "Generic methods require go{}.{}, but go.mod declares go{}.{}",
                GO_GENERIC_METHODS_SINCE.0, GO_GENERIC_METHODS_SINCE.1, declared.0, declared.1
            ),
        });
    }
}

/// Collect `method_declaration` nodes that carry type parameters — i.e. generic
/// methods. Generic *functions* (1.18) are not flagged; only methods are new in
/// 1.27, and they are a distinct grammar node.
fn collect_generic_methods<'a>(node: &Node<'a>, out: &mut Vec<Node<'a>>) {
    if node.kind() == "method_declaration" && node.child_by_field_name("type_parameters").is_some() {
        out.push(*node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_generic_methods(&child, out);
    }
}

/// Walk up from a Go source file to the nearest `go.mod` and read its declared
/// version. Stops at the filesystem root.
pub fn resolve_go_version(file_path: &Path) -> Option<Version> {
    let mut dir = file_path.parent()?;
    loop {
        let go_mod = dir.join("go.mod");
        if let Ok(contents) = std::fs::read_to_string(&go_mod) {
            return parse_go_directive(&contents);
        }
        match dir.parent() {
            Some(parent) if parent != dir => dir = parent,
            _ => return None,
        }
    }
}

/// Parse the `go` directive from go.mod contents.
///
/// Accepts the forms Go itself accepts: `go 1.27`, `go 1.27.0`, `go 1.27rc2`.
/// Only the major and minor numbers are kept.
pub fn parse_go_directive(contents: &str) -> Option<Version> {
    for line in contents.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("go ") else {
            continue;
        };
        let ver = rest.trim();
        // Take the leading "major.minor", ignoring any patch / rc suffix.
        let mut parts = ver.split('.');
        let major: u32 = parts.next()?.parse().ok()?;
        let minor_field = parts.next()?;
        // "27rc2" -> 27 ; "0" -> 0
        let minor_digits: String = minor_field.chars().take_while(|c| c.is_ascii_digit()).collect();
        let minor: u32 = minor_digits.parse().ok()?;
        return Some((major, minor));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_go_directive_variants() {
        assert_eq!(parse_go_directive("module x\n\ngo 1.27\n"), Some((1, 27)));
        assert_eq!(parse_go_directive("go 1.27.0"), Some((1, 27)));
        assert_eq!(parse_go_directive("go 1.27rc2"), Some((1, 27)));
        assert_eq!(parse_go_directive("go 1.21  \n"), Some((1, 21)));
        assert_eq!(parse_go_directive("go 1.27.3 // comment"), Some((1, 27)));
    }

    #[test]
    fn ignores_toolchain_and_other_lines() {
        let go_mod = "module example.com/x\n\ngo 1.26\n\ntoolchain go1.27.0\n";
        // The `go` directive wins, not the toolchain line.
        assert_eq!(parse_go_directive(go_mod), Some((1, 26)));
    }

    #[test]
    fn returns_none_without_a_go_directive() {
        assert_eq!(parse_go_directive("module x\n"), None);
        assert_eq!(parse_go_directive(""), None);
    }

    fn go_issues(src: &str, config: &CodopsyConfig) -> Vec<Issue> {
        use crate::analyzer::ast_utils::parse_source;
        let tree = parse_source(src, SourceLanguage::Go).unwrap();
        let mut issues = Vec::new();
        // Drive the tree-walk + severity directly, bypassing go.mod resolution.
        let severity = config
            .get_rule_severity("go-generic-method")
            .unwrap_or(Severity::Warning);
        let mut methods = Vec::new();
        collect_generic_methods(&tree.root_node(), &mut methods);
        for node in methods {
            issues.push(Issue {
                file: "x.go".to_string(),
                line: node_line(&node),
                column: node_column(&node),
                severity,
                rule: "go-generic-method".to_string(),
                message: String::new(),
            });
        }
        issues
    }

    #[test]
    fn detects_generic_method_only() {
        let cfg = CodopsyConfig::default();
        // Generic method -> flagged.
        let m = "package x
type S[T any] struct{}
func (s S[T]) Map[U any](f func(T) U) {}
";
        assert_eq!(go_issues(m, &cfg).len(), 1);
        // Generic *function* (1.18) -> not flagged.
        let f = "package x
func Map[T, U any](x T) U { var u U; return u }
";
        assert_eq!(go_issues(f, &cfg).len(), 0);
        // Plain method -> not flagged.
        let p = "package x
type S struct{}
func (s S) M() {}
";
        assert_eq!(go_issues(p, &cfg).len(), 0);
    }

    #[test]
    fn version_ordering_is_by_major_then_minor() {
        assert!((1, 26) < GO_GENERIC_METHODS_SINCE);
        assert!((1, 27) >= GO_GENERIC_METHODS_SINCE);
        assert!((2, 0) >= GO_GENERIC_METHODS_SINCE);
    }
}
