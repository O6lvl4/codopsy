/// Node kind classification helpers shared across complexity analysis.
///
/// Node kinds are not globally unique across tree-sitter grammars: `match` is
/// both a Haskell function equation and a Lean 4 pattern match, `instance` is
/// both a Haskell instance head and a Lean instance. Kinds that only ever mean
/// one thing live in the shared tables below; where a kind is contested, the
/// language selects the table.
use tree_sitter::Node;

use crate::analyzer::ast_utils::SourceLanguage;

/// Is this a branching construct for cyclomatic complexity?
pub fn is_cc_increment(kind: &str, lang: SourceLanguage) -> bool {
    if lang == SourceLanguage::Lean {
        return matches!(kind, "if_then_else" | "match_alt" | "by_cases");
    }
    matches!(
        kind,
        // JS/TS / Go / Java / C / C++ / C# / PHP / Scala / Python
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
            // Python
            | "elif_clause"
            | "except_clause"
            | "case_clause"
            // Go
            | "type_case_clause"
            | "communication_case"
            | "select_statement"
            // Ruby
            | "if"
            | "elsif"
            | "unless"
            | "when"
            | "for"
            | "while"
            | "until"
            | "rescue"
            // Haskell
            | "match"
            | "alternative"
            | "guard"
            // Almide
            | "for_in_expression"
            // Erlang
            | "case_expr"
            | "if_expr"
            | "receive_expr"
            | "try_expr"
            | "cr_clause"
            // Gleam: shares "case_clause" with Python
    )
}

/// Is this an if-like node?
pub fn is_if_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_statement" | "if_expression" | "if" | "conditional_expression"
            // Lean 4
            | "if_then_else"
    )
}

/// Is this a nesting construct for cognitive complexity?
pub fn is_nesting_construct(kind: &str, lang: SourceLanguage) -> bool {
    if lang == SourceLanguage::Lean {
        return matches!(kind, "match" | "induction");
    }
    matches!(
        kind,
        // JS/TS / Go / Java / C / C++ / C# / PHP
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
            // Python
            | "except_clause"
            // Go
            | "select_statement"
            // Ruby
            | "for"
            | "while"
            | "until"
            | "unless"
            | "case"
            | "rescue"
            // Almide
            | "for_in_expression"
            | "do_expression"
            // Erlang
            | "case_expr"
            | "receive_expr"
            // Gleam: shares "case" with Ruby
    )
}

/// Is this a break/continue node?
pub fn is_break_continue(kind: &str) -> bool {
    matches!(
        kind,
        "break_statement"
            | "continue_statement"
            | "break_expression"
            | "continue_expression"
            | "break"
    )
}

/// Is this a logical operator?
pub fn is_logical_op(op: &str) -> bool {
    // `∧` / `∨` are Lean 4's `And` / `Or` connectives.
    matches!(op, "&&" | "||" | "??" | "and" | "or" | "∧" | "∨")
}

/// Field names for the (left, operator, right) parts of a binary expression.
/// tree-sitter grammars disagree on the naming; Lean 4 uses lhs/op/rhs.
pub fn binary_op_fields(kind: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match kind {
        "binary_expression" => Some(("left", "operator", "right")),
        "binary_op" => Some(("lhs", "op", "rhs")),
        _ => None,
    }
}

/// Field names for the (condition, consequence, alternative) parts of an if node.
pub fn if_fields(kind: &str) -> (&'static str, &'static str, &'static str) {
    match kind {
        "if_then_else" => ("cond", "then", "else"),
        _ => ("condition", "consequence", "alternative"),
    }
}

/// Operator text of a binary node, if it is a *logical* operator.
/// Returns `None` for non-binary nodes and for arithmetic/comparison operators.
pub fn logical_op_text<'a>(node: &Node, source: &'a [u8]) -> Option<&'a str> {
    let (_, op_field, _) = binary_op_fields(node.kind())?;
    let op = node.child_by_field_name(op_field)?.utf8_text(source).unwrap_or("");
    is_logical_op(op).then_some(op)
}

/// Collect logical operator kinds from a binary expression tree (flattened).
fn collect_logical_ops(node: &Node, source: &[u8], ops: &mut Vec<String>) {
    let Some((left_field, _, right_field)) = binary_op_fields(node.kind()) else { return };
    let Some(op) = logical_op_text(node, source) else { return };
    if let Some(left) = node.child_by_field_name(left_field) {
        collect_logical_ops(&left, source, ops);
    }
    ops.push(op.to_string());
    if let Some(right) = node.child_by_field_name(right_field) {
        collect_logical_ops(&right, source, ops);
    }
}

/// Count the number of distinct adjacent operator "groups" in a logical expression.
pub fn count_logical_op_switches(node: &Node, source: &[u8]) -> usize {
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
pub fn is_top_level_logical(node: &Node, source: &[u8]) -> bool {
    if logical_op_text(node, source).is_none() {
        return false;
    }
    let Some(parent) = node.parent() else { return true };
    logical_op_text(&parent, source).is_none()
}
