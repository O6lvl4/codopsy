use tree_sitter::{Node, Tree};

use crate::types::{ComplexityResult, FunctionComplexity};

use super::ast_utils::{get_function_name, is_function_node, node_line, SourceLanguage};
use super::decl::{
    clojure_defn_name, elixir_def_name, erlang_fun_name, is_clojure_defn, is_elixir_def_call,
    is_erlang_fun_decl, is_lean_instance, lean_instance_name,
};
use super::node_classify::*;

/// Does this node start a new function-sized unit, i.e. should a walk that is
/// scoped to one function stop here?
fn is_unit_boundary(node: &Node, lang: SourceLanguage) -> bool {
    is_function_node(node) || (lang == SourceLanguage::Lean && is_lean_instance(node))
}

/// Calculate cyclomatic complexity for a function node.
fn calculate_cyclomatic(node: &Node, source: &[u8], lang: SourceLanguage) -> usize {
    let mut complexity = 0;

    fn walk(node: &Node, root: &Node, complexity: &mut usize, source: &[u8], lang: SourceLanguage) {
        if node.id() != root.id() && is_unit_boundary(node, lang) {
            return;
        }
        // Anonymous nodes are keyword tokens (and always leaves). Several
        // grammars name them after the construct they introduce — Ruby's `if`
        // token inside an `if` node, Lean's `if` inside `if_then_else` — so
        // classifying them would double-count every branch.
        if !node.is_named() {
            return;
        }
        if is_cc_increment(node.kind(), lang) {
            *complexity += 1;
        }
        if logical_op_text(node, source).is_some() {
            *complexity += 1;
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
            walk(&child, root, complexity, source, lang);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, node, &mut complexity, source, lang);
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

struct CogCtx<'a> {
    func_node_id: usize,
    source: &'a [u8],
    lang: SourceLanguage,
    complexity: usize,
}

impl<'a> CogCtx<'a> {
    fn is_lean(&self) -> bool {
        self.lang == SourceLanguage::Lean
    }

    fn walk(&mut self, node: &Node, nesting: usize) {
        if node.id() != self.func_node_id && is_unit_boundary(node, self.lang) {
            return;
        }
        // See the note in `calculate_cyclomatic`: keyword tokens must not be
        // classified as the construct they introduce.
        if !node.is_named() {
            return;
        }
        let kind = node.kind();

        if is_if_node(kind) {
            self.handle_if(node, nesting);
            return;
        }
        if is_nesting_construct(kind, self.lang) {
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
        self.complexity += if self.is_else_if(node) { 1 } else { 1 + nesting };
        self.score_condition(node);
        self.walk_consequence(node, nesting);
        self.walk_alternative(node, nesting);
    }

    /// An `else if` costs a flat +1 instead of `1 + nesting`. Most grammars wrap
    /// the tail in an `else_clause`; Lean 4 puts the nested `if_then_else`
    /// straight into the `else` field.
    fn is_else_if(&self, node: &Node) -> bool {
        let Some(parent) = node.parent() else { return false };
        if parent.kind() == "else_clause" {
            return true;
        }
        if !self.is_lean() || !is_if_node(parent.kind()) {
            return false;
        }
        let (_, _, else_field) = if_fields(parent.kind());
        parent
            .child_by_field_name(else_field)
            .is_some_and(|alt| alt.id() == node.id())
    }

    fn score_condition(&mut self, node: &Node) {
        let (cond_field, _, _) = if_fields(node.kind());
        let Some(condition) = node.child_by_field_name(cond_field) else { return };
        let expr = if condition.kind() == "parenthesized_expression" {
            condition.child(1).unwrap_or(condition)
        } else {
            condition
        };
        self.complexity += count_logical_op_switches(&expr, self.source);
    }

    fn walk_consequence(&mut self, node: &Node, nesting: usize) {
        let (_, then_field, _) = if_fields(node.kind());
        let Some(consequence) = node.child_by_field_name(then_field) else { return };
        // In Lean the branch *is* the expression, not a block wrapping it.
        if self.is_lean() {
            self.walk(&consequence, nesting + 1);
        } else {
            self.walk_children(&consequence, nesting + 1);
        }
    }

    fn walk_alternative(&mut self, node: &Node, nesting: usize) {
        let (_, _, else_field) = if_fields(node.kind());
        let Some(alternative) = node.child_by_field_name(else_field) else { return };
        if self.is_lean() {
            // An `else if` chain hangs directly off the `else` field.
            if is_if_node(alternative.kind()) {
                self.handle_if(&alternative, nesting);
            } else {
                self.complexity += 1;
                self.walk(&alternative, nesting + 1);
            }
            return;
        }
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

fn calculate_cognitive_complexity(
    func_node: &Node,
    source: &[u8],
    lang: SourceLanguage,
) -> usize {
    let mut ctx = CogCtx {
        func_node_id: func_node.id(),
        source,
        lang,
        complexity: 0,
    };
    ctx.walk_children(func_node, 0);
    ctx.complexity
}

/// Name of the function-sized unit this node introduces, or `None` if it does
/// not introduce one. Grammars that do not model declarations as a dedicated
/// node kind (Elixir, Clojure, Erlang) are recognised by shape instead.
fn unit_name(node: &Node, source: &[u8], lang: SourceLanguage) -> Option<String> {
    if is_function_node(node) {
        Some(get_function_name(node, source))
    } else if is_elixir_def_call(node, source) {
        Some(elixir_def_name(node, source))
    } else if is_clojure_defn(node, source) {
        Some(clojure_defn_name(node, source))
    } else if is_erlang_fun_decl(node) {
        Some(erlang_fun_name(node, source))
    } else if lang == SourceLanguage::Lean && is_lean_instance(node) {
        Some(lean_instance_name(node, source))
    } else {
        None
    }
}

pub fn analyze_complexity(
    tree: &Tree,
    source: &[u8],
    language: SourceLanguage,
) -> ComplexityResult {
    let root = tree.root_node();
    let mut functions = Vec::new();

    fn visit(
        node: &Node,
        source: &[u8],
        lang: SourceLanguage,
        functions: &mut Vec<FunctionComplexity>,
    ) {
        if let Some(name) = unit_name(node, source, lang) {
            functions.push(FunctionComplexity {
                name,
                line: node_line(node),
                complexity: 1 + calculate_cyclomatic(node, source, lang),
                cognitive_complexity: calculate_cognitive_complexity(node, source, lang),
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            visit(&child, source, lang, functions);
        }
    }

    visit(&root, source, language, &mut functions);

    let cyclomatic = functions.iter().map(|f| f.complexity).max().unwrap_or(0);
    let cognitive = functions.iter().map(|f| f.cognitive_complexity).max().unwrap_or(0);

    ComplexityResult { cyclomatic, cognitive, functions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ast_utils::parse_source;

    fn analyze(source: &str, lang: SourceLanguage) -> ComplexityResult {
        let tree = parse_source(source, lang).unwrap();
        analyze_complexity(&tree, source.as_bytes(), lang)
    }

    fn analyze_js(source: &str) -> ComplexityResult {
        analyze(source, SourceLanguage::JavaScript)
    }

    fn analyze_rust(source: &str) -> ComplexityResult {
        analyze(source, SourceLanguage::Rust)
    }

    fn analyze_lean(source: &str) -> ComplexityResult {
        analyze(source, SourceLanguage::Lean)
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

    // --- Lean 4 ---
    #[test]
    fn lean_declaration_baseline() {
        let result = analyze_lean("theorem refl_eq (a : Nat) : a = a := rfl");
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "refl_eq");
        assert_eq!(result.functions[0].complexity, 1);
        assert_eq!(result.functions[0].cognitive_complexity, 0);
    }

    #[test]
    fn lean_if_then_else_counts_once() {
        let result = analyze_lean("def f (n : Nat) : Nat :=\n  if n == 0 then 1 else n");
        assert_eq!(result.functions[0].complexity, 2);
        // `if` +1, `else` +1
        assert_eq!(result.functions[0].cognitive_complexity, 2);
    }

    #[test]
    fn lean_else_if_does_not_nest() {
        let result =
            analyze_lean("def f (n : Nat) : String :=\n  if n == 0 then \"z\" else if n < 10 then \"s\" else \"b\"");
        assert_eq!(result.functions[0].complexity, 3);
        // An `else if` chain stays flat: +1 each, not 1 + nesting.
        assert_eq!(result.functions[0].cognitive_complexity, 3);
    }

    #[test]
    fn lean_match_counts_arms_not_the_match() {
        let result =
            analyze_lean("def f (x : Nat) : Nat :=\n  match x with\n  | 0 => 1\n  | _ => 2");
        // One per arm, like a Rust `match`.
        assert_eq!(result.functions[0].complexity, 3);
        assert_eq!(result.functions[0].cognitive_complexity, 1);
    }

    #[test]
    fn lean_nested_if_inside_match_costs_more() {
        let result = analyze_lean(
            "def f (a b : Nat) : Nat :=\n  match a with\n  | 0 => if b == 0 then 1 else 2\n  | _ => 3",
        );
        assert_eq!(result.functions[0].complexity, 4);
        // match +1, if at nesting 1 → +2, else +1
        assert_eq!(result.functions[0].cognitive_complexity, 4);
    }

    #[test]
    fn lean_logical_operators_count() {
        let result =
            analyze_lean("def f (a b c : Bool) : Bool :=\n  if a && b || c then true else false");
        assert_eq!(result.functions[0].complexity, 4);
        assert_eq!(result.functions[0].cognitive_complexity, 4);
    }

    #[test]
    fn lean_instance_is_its_own_unit() {
        let result = analyze_lean("instance : ToString Nat where\n  toString n := \"n\"");
        assert!(result
            .functions
            .iter()
            .any(|f| f.name == "instance ToString Nat"));
    }

    #[test]
    fn empty_source_no_functions() {
        let result = analyze_js("");
        assert!(result.functions.is_empty());
        assert_eq!(result.cyclomatic, 0);
        assert_eq!(result.cognitive, 0);
    }
}
