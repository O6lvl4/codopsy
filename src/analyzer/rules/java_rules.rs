use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `System.out.println` / `System.err.println` calls.
pub fn check_no_sysout(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "method_invocation" {
            return;
        }
        let Some(obj) = node.child_by_field_name("object") else { return };
        let obj_text = node_text(&obj, ctx.source);
        if matches!(obj_text, "System.out" | "System.err") {
            ctx.report(
                node,
                "no-sysout",
                format!("Avoid `{obj_text}.println()`, use a logging framework"),
            );
        }
    })
}

/// Detect `e.printStackTrace()` calls.
pub fn check_no_print_stack_trace(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "method_invocation" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) == "printStackTrace" {
            ctx.report(
                node,
                "no-print-stack-trace",
                "Avoid `printStackTrace()`, use a logging framework".into(),
            );
        }
    })
}

/// Detect empty catch blocks.
pub fn check_no_empty_catch(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "catch_clause" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() == "block" {
            let mut cursor = body.walk();
            let has_statement = body.children(&mut cursor).any(|c| {
                let k = c.kind();
                !matches!(k, "{" | "}" | "comment" | "line_comment" | "block_comment")
            });
            if !has_statement {
                ctx.report(node, "no-empty-catch", "Empty catch block; at least log the exception".into());
            }
        }
    })
}

/// Detect `throws Exception` (too broad).
pub fn check_no_throws_exception(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "throws" {
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" && node_text(&child, ctx.source) == "Exception" {
                ctx.report(
                    &child,
                    "no-throws-exception",
                    "Avoid `throws Exception`, declare specific exception types".into(),
                );
            }
        }
    })
}

/// Detect `==` comparison on strings (should use `.equals()`).
/// Inspired by Checkstyle's StringLiteralEquality.
pub fn check_no_string_equality(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if op_text != "==" && op_text != "!=" {
            return;
        }
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        if left.kind() == "string_literal" || right.kind() == "string_literal" {
            let suggested = if op_text == "==" { ".equals()" } else { "!.equals()" };
            ctx.report(
                node,
                "no-string-equality",
                format!("Use `{suggested}` instead of `{op_text}` for string comparison"),
            );
        }
    })
}

/// Detect missing default case in switch statements.
/// Inspired by Checkstyle's MissingSwitchDefault.
pub fn check_no_missing_switch_default(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_expression" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        let mut cursor = body.walk();
        let has_default = body.children(&mut cursor).any(|c| {
            if c.kind() != "switch_label" {
                return false;
            }
            let mut inner = c.walk();
            let result = c.children(&mut inner).any(|gc| gc.kind() == "default");
            result
        });
        if !has_default {
            ctx.report(node, "missing-switch-default", "Switch should include a default case".into());
        }
    })
}

/// Detect `new` with raw types (e.g., `new ArrayList()` without generics).
pub fn check_no_raw_type(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "object_creation_expression" {
            return;
        }
        let Some(type_node) = node.child_by_field_name("type") else { return };
        let type_text = node_text(&type_node, ctx.source);
        // Common collection types that should always have generics
        let raw_types = ["ArrayList", "HashMap", "HashSet", "LinkedList", "TreeMap", "TreeSet", "Vector"];
        if raw_types.contains(&type_text) {
            ctx.report(
                node,
                "no-raw-type",
                format!("Use parameterized type instead of raw `{type_text}`"),
            );
        }
    })
}

/// Detect empty if blocks (if_statement where consequence block has no statements).
pub fn check_no_empty_if_java(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() == "block" {
            let mut cursor = body.walk();
            let has_statement = body.children(&mut cursor).any(|c| {
                let k = c.kind();
                !matches!(k, "{" | "}" | "comment" | "line_comment" | "block_comment")
            });
            if !has_statement {
                ctx.report(node, "no-empty-if", "Empty if block".into());
            }
        }
    })
}

/// Detect double-brace initialization pattern: `new ArrayList() {{ add(1); }}`.
/// This creates an anonymous inner class and should be avoided.
pub fn check_no_double_brace_init(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "object_creation_expression" {
            return;
        }
        // Check if it has a class_body (anonymous inner class) with a block
        // (instance initializer). tree-sitter-java represents `{{ ... }}` as
        // class_body > block, not as a dedicated `instance_initializer` node.
        let mut cursor = node.walk();
        let has_init_block = node.children(&mut cursor).any(|c| {
            if c.kind() != "class_body" {
                return false;
            }
            let mut inner = c.walk();
            let result = c.children(&mut inner).any(|gc| {
                matches!(gc.kind(), "block" | "instance_initializer")
            });
            result
        });
        if has_init_block {
            ctx.report(
                node,
                "no-double-brace-init",
                "Avoid double-brace initialization; it creates an anonymous inner class".into(),
            );
        }
    })
}

/// Detect string concatenation using `+` inside loops.
pub fn check_no_string_concat_in_loop(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "binary_expression" {
            return;
        }
        let Some(op) = node.child_by_field_name("operator") else { return };
        let op_text = node_text(&op, ctx.source);
        if op_text != "+" && op_text != "+=" {
            return;
        }
        // Check if either operand is a string literal or string-typed
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else { return };
        let has_string = left.kind() == "string_literal" || right.kind() == "string_literal";
        if !has_string {
            return;
        }
        // Walk up to check if inside a loop
        let mut current = node.parent();
        while let Some(parent) = current {
            if matches!(parent.kind(), "for_statement" | "while_statement" | "enhanced_for_statement") {
                ctx.report(
                    node,
                    "no-string-concat-in-loop",
                    "Avoid string concatenation with `+` inside loops; use StringBuilder".into(),
                );
                return;
            }
            // Stop at function/method boundary
            if matches!(parent.kind(), "method_declaration" | "constructor_declaration" | "lambda_expression") {
                return;
            }
            current = parent.parent();
        }
    })
}

/// Detect try blocks nested inside other try blocks.
pub fn check_no_nested_try(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "try_statement" {
            return;
        }
        // Walk up to check if inside another try_statement
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "try_statement" {
                ctx.report(
                    node,
                    "no-nested-try",
                    "Avoid nested try blocks; extract inner try into a separate method".into(),
                );
                return;
            }
            // Stop at function/method boundary
            if matches!(parent.kind(), "method_declaration" | "constructor_declaration" | "lambda_expression") {
                return;
            }
            current = parent.parent();
        }
    })
}

/// Detect `x.equals(null)` which always returns false or throws NPE.
pub fn check_equals_null(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "method_invocation" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) != "equals" {
            return;
        }
        let Some(args) = node.child_by_field_name("arguments") else { return };
        let mut cursor = args.walk();
        let has_null_arg = args.children(&mut cursor).any(|c| {
            c.kind() == "null_literal" || (c.kind() == "identifier" && node_text(&c, ctx.source) == "null")
        });
        if has_null_arg {
            ctx.report(
                node,
                "equals-null",
                "`x.equals(null)` always returns false; use `x == null` instead".into(),
            );
        }
    })
}
