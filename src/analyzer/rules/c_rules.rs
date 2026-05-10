use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `printf` / `fprintf` / `sprintf` calls (prefer snprintf or a safe alternative).
pub fn check_no_printf(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "printf" | "fprintf" | "sprintf" | "puts") {
            ctx.report(
                node,
                "no-printf",
                format!("Avoid `{name}()` in production code"),
            );
        }
    })
}

/// Detect unsafe functions: `gets`, `strcpy`, `strcat`, `sprintf`.
pub fn check_no_unsafe_fn(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        let (is_unsafe, safe_alt) = match name {
            "gets" => (true, "fgets"),
            "strcpy" => (true, "strncpy"),
            "strcat" => (true, "strncat"),
            "sprintf" => (true, "snprintf"),
            _ => (false, ""),
        };
        if is_unsafe {
            ctx.report(
                node,
                "no-unsafe-fn",
                format!("`{name}()` is unsafe, use `{safe_alt}()` instead"),
            );
        }
    })
}

/// Detect `malloc` without paired `free` (heuristic: flags bare malloc usage).
pub fn check_no_malloc(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        let name = node_text(&func, ctx.source);
        if matches!(name, "malloc" | "calloc" | "realloc") {
            ctx.report(
                node,
                "no-malloc",
                format!("`{name}()` detected; ensure matching `free()` exists"),
            );
        }
    })
}

/// Detect `goto` statements.
pub fn check_no_goto(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "goto_statement" {
            ctx.report(node, "no-goto", "Avoid `goto` statements".into());
        }
    })
}

/// Detect `sizeof(ptr)` which gives pointer size, not array size.
/// Heuristic: flags `sizeof` applied to a single identifier.
pub fn check_no_sizeof_ptr(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "sizeof_expression" {
            return;
        }
        // Look for sizeof(identifier) — the argument is a single identifier
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "sizeof" | "(" | ")"))
            .collect();
        if children.len() == 1 {
            let arg = &children[0];
            // Direct identifier or parenthesized_expression containing just an identifier
            let inner = if arg.kind() == "parenthesized_expression" {
                arg.child(1)
            } else {
                Some(*arg)
            };
            if let Some(inner_node) = inner {
                if inner_node.kind() == "identifier" {
                    let name = node_text(&inner_node, ctx.source);
                    ctx.report(
                        node,
                        "no-sizeof-ptr",
                        format!("`sizeof({name})` returns pointer size if `{name}` is a pointer; consider `sizeof(*{name})` or an explicit size"),
                    );
                }
            }
        }
    })
}

/// Detect magic numbers (numeric literals that aren't 0, 1, or -1) outside of initializations.
pub fn check_no_magic_number(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "number_literal" {
            return;
        }
        let text = node_text(node, ctx.source);
        // Allow 0, 1, -1, 0.0, 1.0
        if matches!(text, "0" | "1" | "0.0" | "1.0" | "0L" | "1L" | "0U" | "1U"
            | "0UL" | "1UL" | "0LL" | "1LL" | "0ULL" | "1ULL") {
            return;
        }
        // Allow if inside init_declarator (variable initialization)
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "init_declarator" {
                return;
            }
            // Also allow in enum values and preprocessor defines
            if matches!(parent.kind(), "enumerator" | "preproc_def") {
                return;
            }
            // Stop searching at statement level
            if parent.kind().ends_with("_statement") || parent.kind() == "declaration" {
                break;
            }
            current = parent.parent();
        }
        ctx.report(
            node,
            "no-magic-number",
            format!("Magic number `{text}`; consider extracting to a named constant"),
        );
    })
}

/// Detect switch case without break/return/goto (implicit fallthrough).
pub fn check_no_implicit_fallthrough_c(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "switch_statement" {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else { return };
        // body is a compound_statement
        let mut cursor = body.walk();
        let children: Vec<_> = body.children(&mut cursor).collect();

        // Collect case indices
        let mut case_indices: Vec<usize> = Vec::new();
        for (i, c) in children.iter().enumerate() {
            if matches!(c.kind(), "case_statement") {
                case_indices.push(i);
            }
        }

        for (idx, &case_i) in case_indices.iter().enumerate() {
            let case_node = &children[case_i];
            // Get the statements within the case_statement
            let mut inner = case_node.walk();
            let stmts: Vec<_> = case_node.children(&mut inner)
                .filter(|c| !matches!(c.kind(), "case" | "default" | ":" | "comment" | "line_comment" | "block_comment"))
                .collect();

            if stmts.is_empty() {
                continue; // Empty case (intentional grouping)
            }

            // Check if the last statement is a terminator
            if let Some(last) = stmts.last() {
                if !ends_with_terminator_c(last) {
                    // Only flag if there is a next case
                    if idx + 1 < case_indices.len() {
                        ctx.report(
                            case_node,
                            "no-implicit-fallthrough",
                            "Case without break, return, or goto before next case".into(),
                        );
                    }
                }
            }
        }
    })
}

fn ends_with_terminator_c(node: &tree_sitter::Node) -> bool {
    let kind = node.kind();
    if matches!(kind, "break_statement" | "return_statement" | "continue_statement" | "goto_statement") {
        return true;
    }
    if kind == "compound_statement" {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        if let Some(last) = children.iter().rev().find(|c| !matches!(c.kind(), "}" | "comment")) {
            return ends_with_terminator_c(last);
        }
    }
    // Recurse into nested case_statement (chained cases)
    if kind == "case_statement" {
        let mut cursor = node.walk();
        let stmts: Vec<_> = node.children(&mut cursor)
            .filter(|c| !matches!(c.kind(), "case" | "default" | ":" | "comment"))
            .collect();
        if let Some(last) = stmts.last() {
            return ends_with_terminator_c(last);
        }
    }
    false
}

/// Detect empty if blocks (if_statement with compound_statement body containing no statements).
pub fn check_no_empty_if_c(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "if_statement" {
            return;
        }
        let Some(body) = node.child_by_field_name("consequence") else { return };
        if body.kind() == "compound_statement" {
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

/// Detect `void main()` which is non-standard in C/C++.
pub fn check_no_void_main(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "function_definition" {
            return;
        }
        let Some(declarator) = node.child_by_field_name("declarator") else { return };
        // Get the function name from the declarator
        let func_name = if declarator.kind() == "function_declarator" {
            declarator.child_by_field_name("declarator")
                .map(|n| node_text(&n, ctx.source))
        } else {
            None
        };
        let Some(name) = func_name else { return };
        if name != "main" {
            return;
        }
        // Check if return type is void
        let Some(type_node) = node.child_by_field_name("type") else { return };
        let type_text = node_text(&type_node, ctx.source);
        if type_text == "void" {
            ctx.report(
                node,
                "no-void-main",
                "`void main()` is non-standard; use `int main()` instead".into(),
            );
        }
    })
}
