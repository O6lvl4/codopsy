use tree_sitter::{Node, Tree};

use crate::types::{ComplexityResult, FunctionComplexity};

use super::ast_utils::{get_function_name, is_function_node, node_line};

/// Is this a branching construct for cyclomatic complexity?
/// Covers both JS/TS and Rust node types.
fn is_cc_increment(kind: &str) -> bool {
    matches!(
        kind,
        // JS/TS
        "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "do_statement"
            | "switch_case"
            | "ternary_expression"
            | "catch_clause"
            // Rust
            | "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_arm"
    )
}

/// Is this a logical operator that increments cyclomatic complexity?
fn is_logical_op(op: &str) -> bool {
    matches!(op, "&&" | "||" | "??")
}

/// Calculate cyclomatic complexity for a function node.
fn calculate_cyclomatic(node: &Node, source: &[u8]) -> usize {
    let mut complexity = 0;

    fn walk(node: &Node, root: &Node, complexity: &mut usize, source: &[u8]) {
        if node.id() != root.id() && is_function_node(node) {
            return;
        }

        let kind = node.kind();

        if is_cc_increment(kind) {
            *complexity += 1;
        }

        if kind == "binary_expression" {
            if let Some(op_node) = node.child_by_field_name("operator") {
                let op = op_node.utf8_text(source).unwrap_or("");
                if is_logical_op(op) {
                    *complexity += 1;
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(&child, root, complexity, source);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, node, &mut complexity, source);
    }

    complexity
}

/// Collect logical operator kinds from a binary expression tree (flattened).
fn collect_logical_ops(node: &Node, source: &[u8], ops: &mut Vec<String>) {
    if node.kind() == "binary_expression" {
        if let Some(op_node) = node.child_by_field_name("operator") {
            let op = op_node.utf8_text(source).unwrap_or("").to_string();
            if is_logical_op(&op) {
                if let Some(left) = node.child_by_field_name("left") {
                    collect_logical_ops(&left, source, ops);
                }
                ops.push(op);
                if let Some(right) = node.child_by_field_name("right") {
                    collect_logical_ops(&right, source, ops);
                }
                return;
            }
        }
    }
}

/// Count the number of distinct adjacent operator "groups" in a logical expression.
fn count_logical_op_switches(node: &Node, source: &[u8]) -> usize {
    let mut ops = Vec::new();
    collect_logical_ops(node, source, &mut ops);
    let mut count = 0;
    let mut prev_op: Option<&str> = None;
    for op in &ops {
        if prev_op != Some(op.as_str()) {
            count += 1;
        }
        prev_op = Some(op.as_str());
    }
    count
}

/// Check if a node is a top-level logical binary expression.
fn is_top_level_logical(node: &Node, source: &[u8]) -> bool {
    if node.kind() != "binary_expression" {
        return false;
    }
    if let Some(op_node) = node.child_by_field_name("operator") {
        let op = op_node.utf8_text(source).unwrap_or("");
        if !is_logical_op(op) {
            return false;
        }
    } else {
        return false;
    }

    if let Some(parent) = node.parent() {
        if parent.kind() == "binary_expression" {
            if let Some(pop) = parent.child_by_field_name("operator") {
                let pop_text = pop.utf8_text(source).unwrap_or("");
                if is_logical_op(pop_text) {
                    return false;
                }
            }
        }
    }

    true
}

/// Is this an if-like node? (JS: if_statement, Rust: if_expression)
fn is_if_node(kind: &str) -> bool {
    kind == "if_statement" || kind == "if_expression"
}

/// Is this a nesting construct for cognitive complexity?
fn is_nesting_construct(kind: &str) -> bool {
    matches!(
        kind,
        // JS/TS
        "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "do_statement"
            | "switch_statement"
            | "catch_clause"
            | "ternary_expression"
            // Rust
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    )
}

/// Is this a break/continue node?
fn is_break_continue(kind: &str) -> bool {
    matches!(
        kind,
        "break_statement"
            | "continue_statement"
            | "break_expression"
            | "continue_expression"
    )
}

struct CogCtx<'a> {
    func_node_id: usize,
    source: &'a [u8],
    complexity: usize,
}

impl<'a> CogCtx<'a> {
    fn new(func_node: &Node, source: &'a [u8]) -> Self {
        Self {
            func_node_id: func_node.id(),
            source,
            complexity: 0,
        }
    }

    fn walk(&mut self, node: &Node, nesting: usize) {
        if node.id() != self.func_node_id && is_function_node(node) {
            return;
        }

        let kind = node.kind();

        if is_if_node(kind) {
            self.handle_if(node, nesting);
            return;
        }

        if is_nesting_construct(kind) {
            self.complexity += 1 + nesting;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.walk(&child, nesting + 1);
            }
            return;
        }

        if is_top_level_logical(node, self.source) {
            self.complexity += count_logical_op_switches(node, self.source);
            return;
        }

        if is_break_continue(kind) && node.child_count() > 1 {
            if let Some(label) = node.child_by_field_name("label") {
                if !label.utf8_text(self.source).unwrap_or("").is_empty() {
                    self.complexity += 1;
                }
            }
        }

        if kind == "optional_chain" {
            self.complexity += 1;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(&child, nesting);
        }
    }

    fn handle_if(&mut self, node: &Node, nesting: usize) {
        let is_else_if = node
            .parent()
            .map(|p| p.kind() == "else_clause")
            .unwrap_or(false);

        if is_else_if {
            self.complexity += 1;
        } else {
            self.complexity += 1 + nesting;
        }

        if let Some(condition) = node.child_by_field_name("condition") {
            let expr = if condition.kind() == "parenthesized_expression" {
                condition.child(1).unwrap_or(condition)
            } else {
                condition
            };
            self.complexity += count_logical_op_switches(&expr, self.source);
        }

        if let Some(consequence) = node.child_by_field_name("consequence") {
            let mut cursor = consequence.walk();
            for child in consequence.children(&mut cursor) {
                self.walk(&child, nesting + 1);
            }
        }

        if let Some(alternative) = node.child_by_field_name("alternative") {
            let mut cursor = alternative.walk();
            let children: Vec<_> = alternative.children(&mut cursor).collect();
            let has_else_if = children.iter().any(|c| is_if_node(c.kind()));

            if has_else_if {
                for child in &children {
                    if is_if_node(child.kind()) {
                        self.handle_if(child, nesting);
                    }
                }
            } else {
                self.complexity += 1;
                for child in &children {
                    self.walk(child, nesting + 1);
                }
            }
        }
    }
}

/// Calculate cognitive complexity for a function node.
fn calculate_cognitive_complexity(func_node: &Node, source: &[u8]) -> usize {
    let mut ctx = CogCtx::new(func_node, source);

    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        ctx.walk(&child, 0);
    }

    ctx.complexity
}

pub fn analyze_complexity(tree: &Tree, source: &[u8]) -> ComplexityResult {
    let root = tree.root_node();
    let mut functions = Vec::new();

    fn visit(node: &Node, source: &[u8], functions: &mut Vec<FunctionComplexity>) {
        if is_function_node(node) {
            let name = get_function_name(node, source);
            let line = node_line(node);
            let cc = 1 + calculate_cyclomatic(node, source);
            let cog = calculate_cognitive_complexity(node, source);
            functions.push(FunctionComplexity {
                name,
                line,
                complexity: cc,
                cognitive_complexity: cog,
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, functions);
        }
    }

    visit(&root, source, &mut functions);

    let cyclomatic = functions.iter().map(|f| f.complexity).max().unwrap_or(0);
    let cognitive = functions
        .iter()
        .map(|f| f.cognitive_complexity)
        .max()
        .unwrap_or(0);

    ComplexityResult {
        cyclomatic,
        cognitive,
        functions,
    }
}
