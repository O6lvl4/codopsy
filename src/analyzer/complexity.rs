use tree_sitter::{Node, Tree};

use crate::types::{ComplexityResult, FunctionComplexity};

use super::ast_utils::{
    clojure_defn_name, elixir_def_name, erlang_fun_name, get_function_name,
    is_clojure_defn, is_elixir_def_call, is_erlang_fun_decl, is_function_node, node_line,
};
use super::node_classify::*;

/// Calculate cyclomatic complexity for a function node.
fn calculate_cyclomatic(node: &Node, source: &[u8]) -> usize {
    let mut complexity = 0;

    fn walk(node: &Node, root: &Node, complexity: &mut usize, source: &[u8]) {
        if node.id() != root.id() && is_function_node(node) {
            return;
        }
        if is_cc_increment(node.kind()) {
            *complexity += 1;
        }
        if node.kind() == "binary_expression" {
            if let Some(op_node) = node.child_by_field_name("operator") {
                if is_logical_op(op_node.utf8_text(source).unwrap_or("")) {
                    *complexity += 1;
                }
            }
        }
        // Clojure: branching forms are list_lit starting with if/cond/case/when/and/or
        if node.kind() == "list_lit" {
            if let Some(first) = first_sym_text(node, source) {
                match first {
                    "if" | "if-let" | "if-not" | "if-some" | "when" | "when-let"
                    | "when-not" | "when-first" => *complexity += 1,
                    "cond" | "condp" | "case" => *complexity += 1,
                    "and" | "or" => *complexity += 1,
                    _ => {}
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

/// Extract the text of the first sym_lit child (for Clojure form detection).
fn first_sym_text<'a>(node: &Node, source: &'a [u8]) -> Option<&'a str> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "sym_lit" {
            return Some(super::ast_utils::node_text(&child, source));
        }
    }
    None
}

/// Collect logical operator kinds from a binary expression tree (flattened).
fn collect_logical_ops(node: &Node, source: &[u8], ops: &mut Vec<String>) {
    if node.kind() != "binary_expression" {
        return;
    }
    let Some(op_node) = node.child_by_field_name("operator") else { return };
    let op = op_node.utf8_text(source).unwrap_or("").to_string();
    if !is_logical_op(&op) {
        return;
    }
    if let Some(left) = node.child_by_field_name("left") {
        collect_logical_ops(&left, source, ops);
    }
    ops.push(op);
    if let Some(right) = node.child_by_field_name("right") {
        collect_logical_ops(&right, source, ops);
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
    let Some(op_node) = node.child_by_field_name("operator") else { return false };
    if !is_logical_op(op_node.utf8_text(source).unwrap_or("")) {
        return false;
    }
    let Some(parent) = node.parent() else { return true };
    if parent.kind() != "binary_expression" {
        return true;
    }
    let Some(pop) = parent.child_by_field_name("operator") else { return true };
    !is_logical_op(pop.utf8_text(source).unwrap_or(""))
}

struct CogCtx<'a> {
    func_node_id: usize,
    source: &'a [u8],
    complexity: usize,
}

impl<'a> CogCtx<'a> {
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
            self.walk_children(node, nesting + 1);
            return;
        }
        if is_top_level_logical(node, self.source) {
            self.complexity += count_logical_op_switches(node, self.source);
            return;
        }
        self.handle_misc(node, kind, nesting);
    }

    fn handle_misc(&mut self, node: &Node, kind: &str, nesting: usize) {
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
        self.walk_children(node, nesting);
    }

    fn handle_if(&mut self, node: &Node, nesting: usize) {
        let is_else_if = node.parent().map_or(false, |p| p.kind() == "else_clause");
        self.complexity += if is_else_if { 1 } else { 1 + nesting };
        self.score_condition(node);
        self.walk_consequence(node, nesting);
        self.walk_alternative(node, nesting);
    }

    fn score_condition(&mut self, node: &Node) {
        let Some(condition) = node.child_by_field_name("condition") else { return };
        let expr = if condition.kind() == "parenthesized_expression" {
            condition.child(1).unwrap_or(condition)
        } else {
            condition
        };
        self.complexity += count_logical_op_switches(&expr, self.source);
    }

    fn walk_consequence(&mut self, node: &Node, nesting: usize) {
        let Some(consequence) = node.child_by_field_name("consequence") else { return };
        self.walk_children(&consequence, nesting + 1);
    }

    fn walk_alternative(&mut self, node: &Node, nesting: usize) {
        let Some(alternative) = node.child_by_field_name("alternative") else { return };
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

    fn walk_children(&mut self, node: &Node, nesting: usize) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk(&child, nesting);
        }
    }
}

fn calculate_cognitive_complexity(func_node: &Node, source: &[u8]) -> usize {
    let mut ctx = CogCtx {
        func_node_id: func_node.id(),
        source,
        complexity: 0,
    };
    ctx.walk_children(func_node, 0);
    ctx.complexity
}

pub fn analyze_complexity(tree: &Tree, source: &[u8]) -> ComplexityResult {
    let root = tree.root_node();
    let mut functions = Vec::new();

    fn visit(node: &Node, source: &[u8], functions: &mut Vec<FunctionComplexity>) {
        if is_function_node(node) {
            functions.push(FunctionComplexity {
                name: get_function_name(node, source),
                line: node_line(node),
                complexity: 1 + calculate_cyclomatic(node, source),
                cognitive_complexity: calculate_cognitive_complexity(node, source),
            });
        } else if is_elixir_def_call(node, source) {
            functions.push(FunctionComplexity {
                name: elixir_def_name(node, source),
                line: node_line(node),
                complexity: 1 + calculate_cyclomatic(node, source),
                cognitive_complexity: calculate_cognitive_complexity(node, source),
            });
        } else if is_clojure_defn(node, source) {
            functions.push(FunctionComplexity {
                name: clojure_defn_name(node, source),
                line: node_line(node),
                complexity: 1 + calculate_cyclomatic(node, source),
                cognitive_complexity: calculate_cognitive_complexity(node, source),
            });
        } else if is_erlang_fun_decl(node) {
            functions.push(FunctionComplexity {
                name: erlang_fun_name(node, source),
                line: node_line(node),
                complexity: 1 + calculate_cyclomatic(node, source),
                cognitive_complexity: calculate_cognitive_complexity(node, source),
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, functions);
        }
    }

    visit(&root, source, &mut functions);

    let cyclomatic = functions.iter().map(|f| f.complexity).max().unwrap_or(0);
    let cognitive = functions.iter().map(|f| f.cognitive_complexity).max().unwrap_or(0);

    ComplexityResult { cyclomatic, cognitive, functions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ast_utils::{parse_source, SourceLanguage};

    fn analyze_js(source: &str) -> ComplexityResult {
        let tree = parse_source(source, SourceLanguage::JavaScript).unwrap();
        analyze_complexity(&tree, source.as_bytes())
    }

    fn analyze_rust(source: &str) -> ComplexityResult {
        let tree = parse_source(source, SourceLanguage::Rust).unwrap();
        analyze_complexity(&tree, source.as_bytes())
    }

    #[test]
    fn simple_function_has_baseline_cc() {
        let result = analyze_js("function foo() { return 1; }");
        // JS parser may produce nested function nodes; verify at least one function found
        assert!(!result.functions.is_empty());
        // CC baseline is at least 1 for any function
        assert!(result.functions.iter().all(|f| f.complexity >= 1));
        // A simple function with no branches has 0 cognitive complexity
        assert!(result.functions.iter().any(|f| f.cognitive_complexity == 0));
    }

    #[test]
    fn if_statement_increments_cc() {
        let result = analyze_js("function foo(x) { if (x) { return 1; } return 0; }");
        let base = analyze_js("function bar() { return 1; }");
        // CC should be higher with an if
        assert!(result.functions[0].complexity > base.functions[0].complexity);
    }

    #[test]
    fn nested_if_increases_cognitive() {
        let result = analyze_js(
            "function foo(a, b) { if (a) { if (b) { return 1; } } return 0; }",
        );
        assert!(result.functions[0].cognitive_complexity >= 3);
    }

    #[test]
    fn rust_function_baseline_cc() {
        let result = analyze_rust("fn main() { let x = 1; }");
        assert_eq!(result.functions.len(), 1);
        assert!(result.functions[0].complexity >= 1);
        assert_eq!(result.functions[0].name, "main");
    }

    #[test]
    fn multiple_functions_detected() {
        let result = analyze_js(
            "function a() {} function b(x) { if (x) {} }",
        );
        assert!(result.functions.len() >= 2);
    }

    #[test]
    fn logical_operators_add_cognitive() {
        let result = analyze_js(
            "function foo(a, b, c) { if (a && b || c) { return 1; } }",
        );
        // if: +1, && group: +1, || switch: +1 = cognitive >= 3
        assert!(result.functions[0].cognitive_complexity >= 3);
    }

    #[test]
    fn empty_source_no_functions() {
        let result = analyze_js("");
        assert!(result.functions.is_empty());
        assert_eq!(result.cyclomatic, 0);
        assert_eq!(result.cognitive, 0);
    }
}
