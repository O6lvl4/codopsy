use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect bare `except:` or `except Exception:` (too-broad exception handling).
pub fn check_no_bare_except(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "except_clause" {
            return;
        }
        // Bare except (no exception type specified)
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        let has_type = children.iter().any(|c| {
            matches!(c.kind(), "identifier" | "tuple" | "as_pattern" | "attribute")
        });
        if !has_type {
            ctx.report(node, "no-bare-except", "Avoid bare `except:`, specify an exception type".into());
            return;
        }
        // `except Exception:` is also too broad
        for child in &children {
            if child.kind() == "identifier" && node_text(child, ctx.source) == "Exception" {
                ctx.report(
                    node,
                    "no-bare-except",
                    "Avoid `except Exception:`, catch specific exceptions".into(),
                );
            }
        }
    })
}

/// Detect `print()` calls (use logging instead).
pub fn check_no_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "print" | "pprint") {
            ctx.report(
                node,
                "no-print",
                format!("Avoid `{name}()`, use the `logging` module"),
            );
        }
    })
}

/// Detect `eval()` and `exec()` calls.
pub fn check_no_eval_exec(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if name == "eval" {
            ctx.report(node, "no-eval", "`eval()` is a security risk".into());
        }
        if name == "exec" {
            ctx.report(node, "no-exec", "`exec()` is a security risk".into());
        }
    })
}

/// Detect mutable default arguments: `def foo(x=[])` or `def foo(x={})`.
pub fn check_no_mutable_default(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "default_parameter" {
            return;
        }
        let Some(value) = node.child_by_field_name("value") else { return };
        if matches!(value.kind(), "list" | "dictionary" | "set") {
            ctx.report(
                node,
                "no-mutable-default",
                "Mutable default argument; use `None` and assign inside the function".into(),
            );
        }
    })
}

/// Detect `global` keyword usage.
pub fn check_no_global(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "global_statement" {
            ctx.report(node, "no-global", "Avoid `global` keyword, pass values explicitly".into());
        }
    })
}

/// Detect unreachable code after return/raise/break/continue.
/// Inspired by Pylint's unreachable.
pub fn check_unreachable(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "block" {
            return;
        }
        let mut found_terminator = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "comment" | "NEWLINE" | "INDENT" | "DEDENT" | "newline") {
                continue;
            }
            if found_terminator {
                ctx.report(&child, "unreachable", "Unreachable code after return/raise".into());
                found_terminator = false;
            }
            if matches!(kind, "return_statement" | "raise_statement" | "break_statement" | "continue_statement") {
                found_terminator = true;
            }
        }
    })
}

/// Detect except blocks that just re-raise (pointless).
/// Inspired by Pylint's pointless-except.
pub fn check_no_pointless_except(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "except_clause" {
            return;
        }
        // Find the body block
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        // Body is typically the last block child
        let body_stmts: Vec<_> = children.iter()
            .filter(|c| !matches!(c.kind(), "except" | ":" | "identifier" | "as_pattern" | "tuple" | "as" | "comment"))
            .collect();

        if body_stmts.len() == 1 {
            let stmt = body_stmts[0];
            // Check if it's a block with just "raise"
            if stmt.kind() == "block" {
                let mut inner_cursor = stmt.walk();
                let inner: Vec<_> = stmt.children(&mut inner_cursor)
                    .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
                    .collect();
                if inner.len() == 1 && inner[0].kind() == "raise_statement" {
                    // Check it's a bare raise (no expression)
                    if inner[0].child_count() <= 1 {
                        ctx.report(node, "pointless-except", "Except block only re-raises; remove the try/except".into());
                    }
                }
            }
        }
    })
}

/// Detect `assert` in non-test code.
pub fn check_no_assert(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    // Only flag if file doesn't look like a test file
    if fp.contains("test_") || fp.contains("_test.") || fp.ends_with("_test.py") {
        return vec![];
    }
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "assert_statement" {
            ctx.report(
                node,
                "no-assert",
                "`assert` is stripped in optimized mode; use explicit error handling".into(),
            );
        }
    })
}

/// Detect class/function with only `pass` as body.
pub fn check_no_pass_body(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if !matches!(node.kind(), "function_definition" | "class_definition") {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if stmts.len() == 1 && stmts[0].kind() == "pass_statement" {
            let what = if node.kind() == "function_definition" { "Function" } else { "Class" };
            ctx.report(
                node,
                "no-pass-body",
                format!("{what} has only `pass` as its body"),
            );
        }
    })
}

/// Detect `from x import *`.
pub fn check_no_star_import(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "import_from_statement" {
            return;
        }
        let mut cursor = node.walk();
        let has_wildcard = node.children(&mut cursor)
            .any(|c| c.kind() == "wildcard_import");
        if has_wildcard {
            ctx.report(
                node,
                "no-star-import",
                "Avoid wildcard imports (`from x import *`); import specific names".into(),
            );
        }
    })
}

/// Detect nested `with` statements that could be combined.
pub fn check_no_nested_with(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "with_statement" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if stmts.len() == 1 && stmts[0].kind() == "with_statement" {
            ctx.report(
                node,
                "no-nested-with",
                "Nested `with` statements can be combined into a single `with`".into(),
            );
        }
    })
}

/// Detect `return value` inside `__init__` method.
pub fn check_no_return_in_init(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_definition" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) != "__init__" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        find_return_with_value(&body, ctx);
    })
}

fn find_return_with_value(node: &tree_sitter::Node, ctx: &mut super::Ctx) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Don't descend into nested function definitions
        if matches!(child.kind(), "function_definition" | "class_definition") {
            continue;
        }
        if child.kind() == "return_statement" {
            // Check if the return has a value (more than just the `return` keyword)
            let mut inner_cursor = child.walk();
            let has_value = child.children(&mut inner_cursor)
                .any(|c| !matches!(c.kind(), "return" | "comment"));
            if has_value {
                ctx.report(
                    &child,
                    "no-return-in-init",
                    "`__init__` should not return a value".into(),
                );
            }
        }
        find_return_with_value(&child, ctx);
    }
}

/// Detect `if cond: return True; else: return False` pattern.
pub fn check_simplify_boolean_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        // Get the consequence block
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        // Get the alternative (else block)
        let Some(alternative) = node.child_by_field_name("alternative") else { return };
        if alternative.kind() != "else_clause" {
            return;
        }

        let then_return = sole_return_bool(&consequence, ctx.source);
        let else_return = sole_return_bool_from_else(&alternative, ctx.source);

        if let (Some(then_val), Some(else_val)) = (then_return, else_return) {
            if (then_val && !else_val) || (!then_val && else_val) {
                ctx.report(
                    node,
                    "simplify-boolean-return",
                    "Simplify `if cond: return True; else: return False` to `return cond`".into(),
                );
            }
        }
    })
}

/// Check if a block contains exactly one return statement returning True or False.
fn sole_return_bool(block: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    if block.kind() != "block" {
        return None;
    }
    let mut cursor = block.walk();
    let stmts: Vec<_> = block.children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
        .collect();
    if stmts.len() != 1 || stmts[0].kind() != "return_statement" {
        return None;
    }
    return_is_bool(&stmts[0], source)
}

/// Extract the boolean from a return statement if it returns True/False.
fn return_is_bool(ret: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    let mut cursor = ret.walk();
    let values: Vec<_> = ret.children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "return" | "comment"))
        .collect();
    if values.len() != 1 {
        return None;
    }
    match node_text(&values[0], source) {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

/// Check if an else_clause body contains exactly one return statement returning True or False.
fn sole_return_bool_from_else(else_clause: &tree_sitter::Node, source: &[u8]) -> Option<bool> {
    let mut cursor = else_clause.walk();
    for child in else_clause.children(&mut cursor) {
        if child.kind() == "block" {
            return sole_return_bool(&child, source);
        }
    }
    None
}

/// Detect `if a: if b:` that can be merged into `if a and b:`.
/// Inspired by Ruff SIM102, Pylint collapsible-if.
pub fn check_collapsible_if_python(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        // Must not have elif or else
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| {
            matches!(c.kind(), "else_clause" | "elif_clause")
        });
        if has_else {
            return;
        }
        // Body must be a block with exactly one if_statement
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut body_cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut body_cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if stmts.len() != 1 || stmts[0].kind() != "if_statement" {
            return;
        }
        // Inner if must also have no else
        let inner = &stmts[0];
        let mut inner_cursor = inner.walk();
        let inner_has_else = inner.children(&mut inner_cursor).any(|c| {
            matches!(c.kind(), "else_clause" | "elif_clause")
        });
        if !inner_has_else {
            ctx.report(node, "collapsible-if", "Nested `if` can be merged: `if a and b:`".into());
        }
    })
}

/// Detect superfluous else after return/raise.
/// Inspired by Ruff RET505.
pub fn check_superfluous_else(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        // Must have else clause
        let mut cursor = node.walk();
        let has_else = node.children(&mut cursor).any(|c| c.kind() == "else_clause");
        if !has_else {
            return;
        }
        // Check if the if-body ends with return or raise
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() != "block" {
            return;
        }
        let mut body_cursor = body.walk();
        let stmts: Vec<_> = body.children(&mut body_cursor)
            .filter(|c| !matches!(c.kind(), "NEWLINE" | "INDENT" | "DEDENT" | "newline" | "comment"))
            .collect();
        if let Some(last) = stmts.last() {
            if matches!(last.kind(), "return_statement" | "raise_statement") {
                ctx.report(node, "superfluous-else", "Remove `else` after `return`/`raise`; dedent the else body".into());
            }
        }
    })
}
