use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

pub fn check_no_any(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "predefined_type" && node_text(node, ctx.source) == "any" {
            ctx.report(node, "no-any", "Unexpected `any` type".into());
        }
    })
}

pub fn check_no_console(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(callee) = node.child_by_field_name("function") else { return };
        if callee.kind() != "member_expression" {
            return;
        }
        let Some(obj) = callee.child_by_field_name("object") else { return };
        if node_text(&obj, ctx.source) == "console" {
            ctx.report(node, "no-console", "Unexpected console statement".into());
        }
    })
}

pub fn check_no_var(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "variable_declaration" {
            return;
        }
        let Some(first) = node.child(0) else { return };
        if node_text(&first, ctx.source) == "var" {
            ctx.report(node, "no-var", "Unexpected var, use let or const instead".into());
        }
    })
}

pub fn check_eqeqeq(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if op_text == "==" || op_text == "!=" {
            let suggested = if op_text == "==" { "===" } else { "!==" };
            ctx.report(&op, "eqeqeq", format!("Expected '{suggested}' instead of '{op_text}'"));
        }
    })
}

pub fn check_no_empty_function(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if !crate::analyzer::ast_utils::is_function_node(node) {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() == "statement_block" && !has_statements(&body) {
            ctx.report(node, "no-empty-function", "Unexpected empty function".into());
        }
    })
}

fn has_statements(block: &Node) -> bool {
    let mut cursor = block.walk();
    let result = block
        .children(&mut cursor)
        .any(|c| c.kind() != "{" && c.kind() != "}" && c.kind() != "comment");
    result
}

pub fn check_no_nested_ternary(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "ternary_expression" && contains_ternary(node) {
            ctx.report(node, "no-nested-ternary", "Do not nest ternary expressions".into());
        }
    })
}

fn contains_ternary(node: &Node) -> bool {
    let mut cursor = node.walk();
    let result = node.children(&mut cursor).any(|c| {
        c.kind() == "ternary_expression" || contains_ternary(&c)
    });
    result
}

/// Detect `import { x as x }` or `const { a: a }` where alias equals original.
/// Inspired by ESLint's no-useless-rename.
pub fn check_no_useless_rename(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "import_specifier" {
            let Some(name) = node.child_by_field_name("name") else { return };
            let Some(alias) = node.child_by_field_name("alias") else { return };
            let name_text = node_text(&name, ctx.source);
            let alias_text = node_text(&alias, ctx.source);
            if name_text == alias_text {
                ctx.report(node, "no-useless-rename", format!("Useless rename: '{name_text}' is renamed to itself"));
            }
        }
        // Detect destructuring `{ a: a }` in variable declarators
        if node.kind() == "pair_pattern" {
            let Some(key) = node.child_by_field_name("key") else { return };
            let Some(value) = node.child_by_field_name("value") else { return };
            if key.kind() == "property_identifier" && value.kind() == "identifier" {
                let key_text = node_text(&key, ctx.source);
                let val_text = node_text(&value, ctx.source);
                if key_text == val_text {
                    ctx.report(node, "no-useless-rename", format!("Useless rename: '{key_text}' is destructured to itself"));
                }
            }
        }
    })
}

/// Detect empty destructuring patterns `const {} = x` or `const [] = x`.
/// Inspired by ESLint's no-empty-pattern.
pub fn check_no_empty_pattern(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let is_target = node.kind() == "object_pattern" || node.kind() == "array_pattern";
        if !is_target {
            return;
        }
        let mut cursor = node.walk();
        let has_content = node.children(&mut cursor).any(|c| {
            !matches!(c.kind(), "{" | "}" | "[" | "]" | "," | "comment")
        });
        if !has_content {
            let kind = if node.kind() == "object_pattern" { "object" } else { "array" };
            ctx.report(node, "no-empty-pattern", format!("Empty {kind} destructuring pattern"));
        }
    })
}

/// Detect constant conditions: `if (true)`, `while (false)`, `x ? true : false`, etc.
/// Inspired by ESLint's no-constant-condition.
pub fn check_no_constant_condition(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let kind = node.kind();
        if !matches!(kind, "if_statement" | "while_statement" | "do_statement" | "ternary_expression"
            | "if_expression" | "while_expression") {
            return;
        }
        let condition = if kind == "ternary_expression" {
            node.child_by_field_name("condition")
        } else {
            node.child_by_field_name("condition")
        };
        let Some(cond) = condition else { return };
        // Unwrap parenthesized_expression
        let inner = if cond.kind() == "parenthesized_expression" {
            cond.child(1).unwrap_or(cond)
        } else {
            cond
        };
        if is_constant_expr(&inner, ctx.source) {
            ctx.report(node, "no-constant-condition", "Unexpected constant condition".into());
        }
    })
}

fn is_constant_expr(node: &Node, source: &[u8]) -> bool {
    match node.kind() {
        "true" | "false" | "null" | "nil" | "none" | "None" => true,
        "number" | "integer" | "float" => true,
        "string" | "template_string" => true,
        "identifier" => {
            let text = node_text(node, source);
            matches!(text, "true" | "false" | "True" | "False" | "nil" | "null" | "None" | "undefined" | "NaN")
        }
        _ => false,
    }
}

/// Detect switch/match without default case.
/// Inspired by ESLint's default-case and Checkstyle's MissingSwitchDefault.
pub fn check_no_missing_default(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_statement" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let has_default = body.children(&mut cursor).any(|c| c.kind() == "switch_default");
        if !has_default {
            ctx.report(node, "default-case", "Switch statement should include a default case".into());
        }
    })
}

/// Detect switch case fallthrough (case without break/return/throw before next case).
/// Inspired by ESLint's no-fallthrough.
pub fn check_no_fallthrough(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_body" {
            return;
        }
        let mut cursor = node.walk();
        let cases: Vec<_> = node.children(&mut cursor)
            .filter(|c| c.kind() == "switch_case")
            .collect();

        for case in &cases {
            let mut inner_cursor = case.walk();
            let children: Vec<_> = case.children(&mut inner_cursor).collect();
            // Skip case keyword and value
            let statements: Vec<_> = children.iter()
                .filter(|c| !matches!(c.kind(), "case" | ":" | "comment" | "line_comment" | "block_comment"))
                .filter(|c| c.child_by_field_name("value").is_none() || c.kind() != "switch_case")
                .collect();

            if statements.is_empty() {
                continue; // Empty case (intentional grouping)
            }

            let last = statements.last();
            if let Some(last_stmt) = last {
                let terminates = ends_with_terminator(last_stmt);
                if !terminates {
                    ctx.report(case, "no-fallthrough", "Expected break, return, or throw before next case".into());
                }
            }
        }
    })
}

fn ends_with_terminator(node: &Node) -> bool {
    let kind = node.kind();
    if matches!(kind, "break_statement" | "return_statement" | "throw_statement" | "continue_statement") {
        return true;
    }
    // Check last child recursively (for block statements)
    if kind == "statement_block" || kind == "block" {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        if let Some(last) = children.iter().rev().find(|c| !matches!(c.kind(), "}" | "comment")) {
            return ends_with_terminator(last);
        }
    }
    false
}

/// Detect `x === x` or `x == x` (self-comparison, likely a bug).
/// Inspired by ESLint's no-self-compare.
pub fn check_no_self_compare(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
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
        if node_text(&left, ctx.source) == node_text(&right, ctx.source)
            && left.kind() == "identifier"
        {
            ctx.report(node, "no-self-compare", format!("Comparing '{}' to itself", node_text(&left, ctx.source)));
        }
    })
}
