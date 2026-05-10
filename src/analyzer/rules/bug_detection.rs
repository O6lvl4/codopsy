use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

pub fn check_no_debugger(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "debugger_statement" {
            ctx.report(node, "no-debugger", "Unexpected 'debugger' statement".into());
        }
    })
}

pub fn check_no_duplicate_case(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_body" {
            return;
        }
        let mut seen = std::collections::HashSet::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "switch_case" {
                continue;
            }
            let Some(value) = child.child_by_field_name("value") else { continue };
            let text = node_text(&value, ctx.source).to_string();
            if !seen.insert(text.clone()) {
                ctx.report(&child, "no-duplicate-case", format!("Duplicate case label: {text}"));
            }
        }
    })
}

pub fn check_no_self_assign(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "assignment_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        if node_text(&op, ctx.source) != "=" {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        let lt = node_text(&left, ctx.source);
        let rt = node_text(&right, ctx.source);
        if lt == rt {
            ctx.report(node, "no-self-assign", format!("'{lt}' is assigned to itself"));
        }
    })
}

pub fn check_no_eval(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(callee) = node.child_by_field_name("function") else { return };
        if callee.kind() == "identifier" && node_text(&callee, ctx.source) == "eval" {
            ctx.report(node, "no-eval", "eval() is not allowed".into());
        }
    })
}

/// Detect try/catch where catch block only re-throws the caught error.
/// Inspired by ESLint's no-useless-catch.
pub fn check_no_useless_catch(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "catch_clause" {
            return;
        }
        // Get the catch parameter name
        let Some(param) = node.child_by_field_name("parameter") else { return };
        let param_text = node_text(&param, ctx.source);
        // Get the catch body
        let Some(body) = node.child_by_field_name("body") else { return };
        // Body should be a statement_block with exactly one statement: a throw_statement
        let mut cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut cursor)
            .filter(|c| c.kind() != "{" && c.kind() != "}" && c.kind() != "comment")
            .collect();
        if stmts.len() != 1 {
            return;
        }
        let stmt = &stmts[0];
        if stmt.kind() != "throw_statement" {
            return;
        }
        // The thrown expression should be the catch parameter
        let mut inner_cursor = stmt.walk();
        let thrown: Vec<_> = stmt.children(&mut inner_cursor)
            .filter(|c| c.kind() != "throw" && c.kind() != ";")
            .collect();
        if thrown.len() == 1 && thrown[0].kind() == "identifier" && node_text(&thrown[0], ctx.source) == param_text {
            ctx.report(node, "no-useless-catch", "Unnecessary catch clause that only re-throws the caught error".into());
        }
    })
}

/// Detect `x === NaN` or `x == NaN` comparisons. Use `Number.isNaN()` instead.
/// Inspired by ESLint's use-isnan.
pub fn check_use_isnan(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if !matches!(op_text, "===" | "==" | "!==" | "!=") {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        let is_nan = |n: &tree_sitter::Node| n.kind() == "identifier" && node_text(n, ctx.source) == "NaN";
        if is_nan(&left) || is_nan(&right) {
            ctx.report(node, "use-isnan", "Use Number.isNaN() instead of comparison with NaN".into());
        }
    })
}

/// Detect `x === -0` comparisons. Use `Object.is(x, -0)` instead.
/// Inspired by ESLint's no-compare-neg-zero.
pub fn check_no_compare_neg_zero(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if !matches!(op_text, "===" | "==" | "!==" | "!=" | ">" | "<" | ">=" | "<=") {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        let is_neg_zero = |n: &tree_sitter::Node| {
            if n.kind() != "unary_expression" {
                return false;
            }
            let Some(op) = n.child_by_field_name("operator") else { return false };
            let Some(arg) = n.child_by_field_name("argument") else { return false };
            node_text(&op, ctx.source) == "-" && arg.kind() == "number" && node_text(&arg, ctx.source) == "0"
        };
        if is_neg_zero(&left) || is_neg_zero(&right) {
            ctx.report(node, "no-compare-neg-zero", "Do not compare against -0, use Object.is(x, -0) instead".into());
        }
    })
}

/// Detect `!key in obj` or `!key instanceof Cls` (likely meant `!(key in obj)`).
/// Inspired by ESLint's no-unsafe-negation.
pub fn check_no_unsafe_negation(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if !matches!(op_text, "in" | "instanceof") {
            return;
        }
        let Some(left) = node.child_by_field_name("left") else { return };
        if left.kind() == "unary_expression" {
            let Some(unary_op) = left.child_by_field_name("operator") else { return };
            if node_text(&unary_op, ctx.source) == "!" {
                ctx.report(node, "no-unsafe-negation", format!("Unexpected negation of left operand of '{op_text}' operator"));
            }
        }
    })
}

/// Detect return statements with a value inside constructor methods.
/// Inspired by ESLint's no-constructor-return.
pub fn check_no_constructor_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "return_statement" {
            return;
        }
        // Only flag returns that have an argument (value)
        let mut cursor = node.walk();
        let has_value = node.children(&mut cursor)
            .any(|c| c.kind() != "return" && c.kind() != ";");
        if !has_value {
            return;
        }
        // Walk up to see if we're inside a constructor method_definition
        let mut current = node.parent();
        while let Some(n) = current {
            if n.kind() == "method_definition" {
                let Some(name) = n.child_by_field_name("name") else { break };
                if node_text(&name, ctx.source) == "constructor" {
                    ctx.report(node, "no-constructor-return", "Unexpected return statement in constructor".into());
                }
                break;
            }
            // Stop if we hit another function boundary
            if matches!(n.kind(), "function_declaration" | "function" | "arrow_function" | "function_expression") {
                break;
            }
            current = n.parent();
        }
    })
}

/// Detect typeof comparisons with invalid type strings.
/// Inspired by ESLint's valid-typeof.
pub fn check_valid_typeof(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if !matches!(op_text, "===" | "==" | "!==" | "!=") {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        let is_typeof = |n: &tree_sitter::Node| {
            // tree-sitter-javascript: typeof is a unary_expression with operator "typeof"
            // tree-sitter-typescript: typeof is a typeof_expression
            n.kind() == "typeof_expression"
                || (n.kind() == "unary_expression" && {
                    n.child_by_field_name("operator")
                        .map_or(false, |op| node_text(&op, ctx.source) == "typeof")
                })
        };
        let get_string_value = |n: &tree_sitter::Node| -> Option<String> {
            if n.kind() == "string" {
                let raw = node_text(n, ctx.source);
                // Strip surrounding quotes
                let trimmed = raw.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                Some(trimmed.to_string())
            } else {
                None
            }
        };
        let valid_types = ["undefined", "object", "boolean", "number", "string", "function", "symbol", "bigint"];
        let check_pair = |typeof_side: &tree_sitter::Node, other_side: &tree_sitter::Node| {
            if is_typeof(typeof_side) {
                if let Some(val) = get_string_value(other_side) {
                    if !valid_types.contains(&val.as_str()) {
                        return true;
                    }
                }
            }
            false
        };
        if check_pair(&left, &right) || check_pair(&right, &left) {
            ctx.report(node, "valid-typeof", "Invalid typeof comparison value".into());
        }
    })
}
