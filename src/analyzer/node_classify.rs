/// Node kind classification helpers shared across complexity analysis.

/// Is this a branching construct for cyclomatic complexity?
pub fn is_cc_increment(kind: &str) -> bool {
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
    )
}

/// Is this an if-like node?
pub fn is_if_node(kind: &str) -> bool {
    matches!(kind, "if_statement" | "if_expression" | "if" | "conditional_expression")
}

/// Is this a nesting construct for cognitive complexity?
pub fn is_nesting_construct(kind: &str) -> bool {
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
    matches!(op, "&&" | "||" | "??" | "and" | "or")
}
