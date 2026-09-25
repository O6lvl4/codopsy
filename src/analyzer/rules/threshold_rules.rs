use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::{is_function_node, node_column, node_line};
use crate::analyzer::node_classify::is_else_if;
use crate::types::{Issue, Severity};

fn effective_line_count(source: &[u8]) -> usize {
    let source_str = std::str::from_utf8(source).unwrap_or("");
    for (i, line) in source_str.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "#[cfg(test)]" || trimmed.starts_with("#[cfg(test)]") {
            return i;
        }
    }
    source.iter().filter(|&&b| b == b'\n').count() + 1
}

pub fn check_max_lines(
    _tree: &Tree,
    source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let line_count = effective_line_count(source);
    if line_count > max {
        vec![Issue {
            file: file_path.to_string(),
            line: 1,
            column: 1,
            severity,
            rule: "max-lines".to_string(),
            message: format!("File has {line_count} lines (max: {max})"),
        }]
    } else {
        vec![]
    }
}

fn is_depth_increment(kind: &str) -> bool {
    matches!(
        kind,
        // JS/TS / Go / Java / C / C++ / C# / PHP
        "if_statement" | "for_statement" | "for_in_statement"
            | "while_statement" | "do_statement" | "switch_statement"
            // Rust
            | "if_expression" | "for_expression" | "while_expression"
            | "loop_expression" | "match_expression"
            // Python
            | "elif_clause"
            // Ruby
            | "if" | "unless" | "for" | "while" | "until" | "case"
            // Go
            | "select_statement"
            // Lean 4
            | "if_then_else" | "match_alt"
    )
}

struct DepthCtx<'a> {
    file_path: &'a str,
    severity: Severity,
    max: usize,
    issues: Vec<Issue>,
}

impl<'a> DepthCtx<'a> {
    fn walk(&mut self, node: &Node, depth: usize) {
        let kind = node.kind();
        // Keyword tokens carry the same kind as the construct they open in
        // several grammars (Ruby's `if`, Lean's `if`); only the node nests.
        // An `else if` continues its chain at the same depth, as ESLint's
        // max-depth counts it: `if a {} else if b {} else if c {}` is one
        // level, however long the chain.
        let increases = node.is_named() && is_depth_increment(kind) && !is_else_if(node);

        let new_depth = if increases { depth + 1 } else { depth };

        if increases && new_depth > self.max {
            self.issues.push(Issue {
                file: self.file_path.to_string(),
                line: node_line(node),
                column: node_column(node),
                severity: self.severity,
                rule: "max-depth".to_string(),
                message: format!("Nesting depth {new_depth} exceeds max of {}", self.max),
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(&child, new_depth);
        }
    }
}

pub fn check_max_depth(
    tree: &Tree,
    _source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let mut ctx = DepthCtx {
        file_path,
        severity,
        max,
        issues: Vec::new(),
    };
    ctx.walk(&tree.root_node(), 0);
    ctx.issues
}

struct ParamsCtx<'a> {
    file_path: &'a str,
    severity: Severity,
    max: usize,
    issues: Vec<Issue>,
}

impl<'a> ParamsCtx<'a> {
    fn visit(&mut self, node: &Node) {
        if is_function_node(node) {
            if let Some(count) = param_count(node) {
                if count > self.max {
                    self.issues.push(Issue {
                        file: self.file_path.to_string(),
                        line: node_line(node),
                        column: node_column(node),
                        severity: self.severity,
                        rule: "max-params".to_string(),
                        message: format!("Function has {count} parameters (max: {})", self.max),
                    });
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(&child);
        }
    }
}

pub fn check_max_params(
    tree: &Tree,
    _source: &[u8],
    file_path: &str,
    severity: Severity,
    max: usize,
) -> Vec<Issue> {
    let mut ctx = ParamsCtx {
        file_path,
        severity,
        max,
        issues: Vec::new(),
    };
    ctx.visit(&tree.root_node());
    ctx.issues
}

/// Number of declared parameters, or `None` if the grammar does not expose them
/// for this node.
fn param_count(node: &Node) -> Option<usize> {
    if let Some(params) = node.child_by_field_name("parameters") {
        return Some(count_params(&params));
    }
    lean_param_count(node)
}

/// Lean 4 spells parameters as `binders`, and one binder group can introduce
/// several of them (`(a b : Nat)` is two). Implicit and instance binders are
/// solved by unification rather than passed by the caller, so they don't count.
///
/// Only value-level declarations are measured: a `theorem` with many hypotheses
/// is normal, not a smell.
fn lean_param_count(node: &Node) -> Option<usize> {
    if !matches!(node.kind(), "def" | "abbrev" | "where_aux_def") {
        return None;
    }
    let mut cursor = node.walk();
    let binders = node.children(&mut cursor).find(|c| c.kind() == "binders")?;
    let mut cursor = binders.walk();
    let count = binders
        .children(&mut cursor)
        .map(|binder| match binder.kind() {
            "explicit_binder" | "anon_ctor_binder" | "tuple_binder" => {
                let mut c = binder.walk();
                binder.children_by_field_name("name", &mut c).count()
            }
            "identifier" | "hole" => 1,
            _ => 0,
        })
        .sum();
    Some(count)
}

fn count_params(params: &Node) -> usize {
    let mut cursor = params.walk();
    params
        .children(&mut cursor)
        .filter(|c| {
            !matches!(c.kind(), "(" | ")" | "," | "|" | "comment" | "line_comment" | "block_comment")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depth_issues(source: &str, language: tree_sitter::Language, max: usize) -> Vec<Issue> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(source, None).unwrap();
        check_max_depth(&tree, source.as_bytes(), "t", Severity::Warning, max)
    }

    #[test]
    fn an_else_if_chain_is_one_level_deep() {
        let js = "function f(a) { if (a == 1) { x() } else if (a == 2) { y() } else if (a == 3) { z() } else { w() } }";
        assert!(depth_issues(js, tree_sitter_javascript::LANGUAGE.into(), 1).is_empty());
        let almide = "fn f(a: Int) -> Int =\n  if a == 1 then 1\n  else if a == 2 then 2\n  else if a == 3 then 3\n  else 4\n";
        assert!(depth_issues(almide, tree_sitter_almide::LANGUAGE.into(), 1).is_empty());
    }

    #[test]
    fn an_if_inside_a_branch_still_nests() {
        let almide = "fn f(a: Int, b: Int) -> Int =\n  if a == 1 then (if b == 1 then 1 else 2)\n  else 3\n";
        assert_eq!(depth_issues(almide, tree_sitter_almide::LANGUAGE.into(), 1).len(), 1);
        let js = "function f(a, b) { if (a) { if (b) { x() } } }";
        assert_eq!(depth_issues(js, tree_sitter_javascript::LANGUAGE.into(), 1).len(), 1);
    }
}
