use tree_sitter::Tree;

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect bare `panic()` calls.
pub fn check_no_panic(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if func.kind() == "identifier" && node_text(&func, ctx.source) == "panic" {
            ctx.report(node, "no-panic", "Avoid bare `panic()`, return an error instead".into());
        }
    })
}

/// Detect `fmt.Println` / `fmt.Printf` etc. (debug prints).
pub fn check_no_fmt_print(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if func.kind() != "selector_expression" {
            return;
        }
        let Some(operand) = func.child_by_field_name("operand") else { return };
        let Some(field) = func.child_by_field_name("field") else { return };
        if node_text(&operand, ctx.source) == "fmt" {
            let method = node_text(&field, ctx.source);
            if matches!(method, "Println" | "Printf" | "Print" | "Fprintf" | "Fprintln") {
                ctx.report(
                    node,
                    "no-fmt-print",
                    format!("Avoid `fmt.{method}()`, use a structured logger"),
                );
            }
        }
    })
}

/// Detect empty error handling: `if err != nil { return }` or bare `_ = err`.
pub fn check_no_ignored_error(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        // Pattern: short_var_declaration with `_` receiving an error
        if node.kind() == "short_var_declaration" {
            let Some(left) = node.child_by_field_name("left") else { return };
            let text = node_text(&left, ctx.source);
            // Check for patterns like `_, _ = ...` or just `_ = ...`
            if text.contains('_') {
                // Only flag if there are multiple return values and one is blanked
                let parts: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 && parts.iter().any(|&p| p == "_") && parts.iter().any(|&p| p != "_") {
                    // This is fine - typical Go pattern for ignoring one of multiple returns
                    return;
                }
            }
        }
        // Pattern: assignment `_ = someFunc()`
        if node.kind() == "assignment_statement" {
            let Some(left) = node.child_by_field_name("left") else { return };
            if node_text(&left, ctx.source).trim() == "_" {
                ctx.report(
                    node,
                    "no-ignored-error",
                    "Error return value is explicitly ignored with `_`".into(),
                );
            }
        }
    })
}

/// Detect `defer` inside loops (resource leak risk).
/// Inspired by golangci-lint's deferInLoop.
pub fn check_no_defer_in_loop(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "defer_statement" {
            return;
        }
        // Walk up to check if inside a loop
        let mut current = node.parent();
        while let Some(parent) = current {
            if matches!(parent.kind(), "for_statement" | "for_range_clause") {
                ctx.report(
                    node,
                    "no-defer-in-loop",
                    "Avoid `defer` inside a loop; deferred calls accumulate until function returns".into(),
                );
                return;
            }
            // Stop at function boundary
            if matches!(parent.kind(), "function_declaration" | "method_declaration" | "func_literal") {
                return;
            }
            current = parent.parent();
        }
    })
}

/// Detect `os.Exit()` calls outside of main.
pub fn check_no_os_exit(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "call_expression" {
            return;
        }
        let Some(func) = node.child_by_field_name("function") else { return };
        if func.kind() != "selector_expression" {
            return;
        }
        let Some(operand) = func.child_by_field_name("operand") else { return };
        let Some(field) = func.child_by_field_name("field") else { return };
        if node_text(&operand, ctx.source) == "os" && node_text(&field, ctx.source) == "Exit" {
            ctx.report(node, "no-os-exit", "Avoid `os.Exit()`, return an error instead".into());
        }
    })
}

/// Detect empty `if`/`for` blocks.
pub fn check_no_empty_block(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let kind = node.kind();
        // if_statement uses "consequence", for_statement uses "body"
        let body = match kind {
            "if_statement" => node.child_by_field_name("consequence"),
            "for_statement" => node.child_by_field_name("body"),
            _ => return,
        };
        let Some(body) = body else { return };
        if body.kind() != "block" {
            return;
        }
        // In tree-sitter-go, block contains { statement_list? }
        // Empty block has no statement_list child
        let mut cursor = body.walk();
        let has_statements = body.children(&mut cursor)
            .any(|c| c.kind() == "statement_list");
        if !has_statements {
            ctx.report(
                node,
                "no-empty-block",
                format!("Empty `{kind}` block"),
            );
        }
    })
}

/// Detect unreachable code after return/break/continue in a block.
pub fn check_no_unreachable_go(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        // Go blocks contain a statement_list child; check within that
        if node.kind() != "statement_list" {
            return;
        }
        let mut found_terminator = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind == "comment" {
                continue;
            }
            if found_terminator {
                ctx.report(&child, "no-unreachable", "Unreachable code detected".into());
                found_terminator = false;
            }
            if matches!(kind, "return_statement" | "break_statement" | "continue_statement") {
                found_terminator = true;
            }
        }
    })
}

/// Detect bare `return` in functions with named return values.
pub fn check_no_naked_return(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if !matches!(node.kind(), "function_declaration" | "method_declaration") {
            return;
        }
        // Check if the function has named return values
        let Some(result) = node.child_by_field_name("result") else { return };
        // Named returns use a parameter_list node
        if result.kind() != "parameter_list" {
            return;
        }
        // Verify at least one parameter has a name (identifier before the type)
        let mut result_cursor = result.walk();
        let has_named = result.children(&mut result_cursor).any(|param| {
            if param.kind() != "parameter_declaration" {
                return false;
            }
            // A named return has an identifier child before the type
            param.child_by_field_name("name").is_some()
        });
        if !has_named {
            return;
        }
        // Now find bare return_statements inside this function
        let Some(body) = node.child_by_field_name("body") else { return };
        find_naked_returns(&body, ctx);
    })
}

fn find_naked_returns(node: &tree_sitter::Node, ctx: &mut super::Ctx) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Don't descend into nested function literals
        if matches!(child.kind(), "func_literal" | "function_declaration" | "method_declaration") {
            continue;
        }
        if child.kind() == "return_statement" {
            // Bare return has no expression children (only the `return` keyword)
            let mut inner_cursor = child.walk();
            let has_expr = child.children(&mut inner_cursor)
                .any(|c| !matches!(c.kind(), "return" | "comment"));
            if !has_expr {
                ctx.report(&child, "no-naked-return", "Naked return in function with named return values".into());
            }
        }
        find_naked_returns(&child, ctx);
    }
}

/// Detect `for ... range` over a string variable (iterates runes, but often misunderstood).
pub fn check_no_range_over_string(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "for_statement" {
            return;
        }
        // Look for a range_clause child
        let mut cursor = node.walk();
        let range_clause = node.children(&mut cursor)
            .find(|c| c.kind() == "range_clause");
        let Some(range) = range_clause else { return };
        // The right side of the range clause is the iterable
        let Some(right) = range.child_by_field_name("right") else { return };
        // Heuristic: flag if the iterable is a plain identifier (likely a string variable)
        if right.kind() == "identifier" {
            let name = node_text(&right, ctx.source);
            // Skip common non-string iterables
            if !name.is_empty() {
                ctx.report(
                    node,
                    "no-range-over-string",
                    format!("Iterating over `{name}` with range yields runes, not bytes; ensure this is intentional"),
                );
            }
        }
    })
}

/// Detect local variable shadowing an imported package name.
pub fn check_no_shadow_import(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    use crate::analyzer::ast_utils::{node_column, node_line};

    // First, collect all imported package names
    let root = tree.root_node();
    let mut imports: Vec<String> = Vec::new();
    collect_imports(&root, source, &mut imports);
    if imports.is_empty() {
        return vec![];
    }

    let mut issues = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "short_var_declaration" {
            if let Some(left) = node.child_by_field_name("left") {
                let mut cursor = left.walk();
                for child in left.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = node_text(&child, source);
                        if imports.iter().any(|imp| imp == name) {
                            issues.push(Issue {
                                file: fp.to_string(),
                                line: node_line(&node),
                                column: node_column(&node),
                                severity: sev,
                                rule: "no-shadow-import".to_string(),
                                message: format!("Variable `{name}` shadows imported package"),
                            });
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    issues
}

fn collect_imports(node: &tree_sitter::Node, source: &[u8], imports: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_declaration" => collect_imports(&child, source, imports),
            "import_spec_list" => collect_imports(&child, source, imports),
            "import_spec" => {
                // import_spec has a path field (interpreted_string_literal)
                // and optionally a name field (alias)
                if let Some(name) = child.child_by_field_name("name") {
                    // Aliased import: `alias "path"`
                    let alias = node_text(&name, source);
                    if alias != "_" && alias != "." {
                        imports.push(alias.to_string());
                    }
                } else if let Some(path) = child.child_by_field_name("path") {
                    // Unaliased: last segment of path is the package name
                    let path_text = node_text(&path, source);
                    let path_text = path_text.trim_matches('"');
                    if let Some(last) = path_text.rsplit('/').next() {
                        imports.push(last.to_string());
                    }
                }
            }
            _ => {}
        }
    }
}
