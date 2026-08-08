//! Lean 4 lint rules.
//!
//! The rules target the two things that make a Lean file untrustworthy rather
//! than merely ugly: proofs that are not actually finished, and declarations
//! that widen what the kernel has to take on faith.

use tree_sitter::{Node, Tree};

use crate::analyzer::ast_utils::node_text;
use crate::types::{Issue, Severity};

use super::run_check;

/// Detect `sorry` — a placeholder that closes any goal and leaves the
/// declaration unproved (Lean marks the whole file as containing `sorryAx`).
pub fn check_no_sorry(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "sorry" && node.is_named() {
            ctx.report(
                node,
                "no-sorry",
                "`sorry` leaves this declaration unproved".into(),
            );
        }
    })
}

/// Detect `axiom` declarations, which extend the set of assumptions the whole
/// development rests on.
pub fn check_no_axiom(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "axiom" && node.is_named() {
            ctx.report(
                node,
                "no-axiom",
                "`axiom` adds an unproved assumption; prove it as a `theorem` instead".into(),
            );
        }
    })
}

/// Detect the `native_decide` tactic, which discharges a goal by trusting the
/// compiler and the runtime in addition to the kernel.
pub fn check_no_native_decide(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "identifier" && node_text(node, ctx.source) == "native_decide" {
            ctx.report(
                node,
                "no-native-decide",
                "`native_decide` trusts the compiler; prefer `decide` or an explicit proof".into(),
            );
        }
    })
}

/// Detect `partial def`, which opts out of the termination checker.
pub fn check_no_partial_def(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if has_modifier(node, ctx.source, "partial") {
            ctx.report(
                node,
                "no-partial-def",
                "`partial` skips the termination checker; add a `termination_by` clause if you can"
                    .into(),
            );
        }
    })
}

/// Detect the `unsafe` modifier, which bypasses the type-safety guarantees the
/// rest of the file relies on.
pub fn check_no_unsafe(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if has_modifier(node, ctx.source, "unsafe") {
            ctx.report(
                node,
                "no-unsafe",
                "`unsafe` bypasses Lean's soundness guarantees".into(),
            );
        }
    })
}

/// `decl_modifiers` is a run of keyword tokens (`private partial`, …).
fn has_modifier(node: &Node, source: &[u8], modifier: &str) -> bool {
    node.kind() == "decl_modifiers"
        && node_text(node, source).split_whitespace().any(|w| w == modifier)
}

/// Detect `dbg_trace`, Lean's print-debugging escape hatch.
pub fn check_no_dbg_trace(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() == "identifier" && node_text(node, ctx.source) == "dbg_trace" {
            ctx.report(
                node,
                "no-dbg-trace",
                "`dbg_trace` is debug output; remove it before committing".into(),
            );
        }
    })
}

/// Detect `#eval` / `#check` / `#print` / `#reduce` commands left in a file.
/// They run at elaboration time and are usually leftovers from exploration.
pub fn check_no_debug_command(tree: &Tree, source: &[u8], fp: &str, sev: Severity) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        let command = match node.kind() {
            "eval" => "#eval",
            "check" => "#check",
            "print" => "#print",
            "reduce" => "#reduce",
            _ => return,
        };
        if !node.is_named() {
            return;
        }
        ctx.report(
            node,
            "no-debug-command",
            format!("`{command}` is an interactive command; remove it before committing"),
        );
    })
}

/// Detect `set_option maxHeartbeats 0`, which removes the elaboration timeout
/// and lets a single declaration hang the build indefinitely.
pub fn check_no_unlimited_heartbeats(
    tree: &Tree,
    source: &[u8],
    fp: &str,
    sev: Severity,
) -> Vec<Issue> {
    run_check(tree, source, fp, sev, |node, ctx| {
        if node.kind() != "set_option" {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else { return };
        if node_text(&name, ctx.source) != "maxHeartbeats" {
            return;
        }
        let Some(value) = node.child_by_field_name("value") else { return };
        if node_text(&value, ctx.source).trim() != "0" {
            return;
        }
        ctx.report(
            node,
            "no-unlimited-heartbeats",
            "`set_option maxHeartbeats 0` removes the elaboration timeout; raise the limit instead"
                .into(),
        );
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ast_utils::{parse_source, SourceLanguage};

    fn rules(src: &str, check: fn(&Tree, &[u8], &str, Severity) -> Vec<Issue>) -> Vec<Issue> {
        let tree = parse_source(src, SourceLanguage::Lean).unwrap();
        check(&tree, src.as_bytes(), "Test.lean", Severity::Warning)
    }

    #[test]
    fn detects_sorry() {
        assert_eq!(rules("theorem t : True := sorry", check_no_sorry).len(), 1);
        assert!(rules("theorem t : True := trivial", check_no_sorry).is_empty());
    }

    #[test]
    fn detects_axiom() {
        assert_eq!(rules("axiom choice : True", check_no_axiom).len(), 1);
        assert!(rules("theorem choice : True := trivial", check_no_axiom).is_empty());
    }

    #[test]
    fn detects_native_decide() {
        assert_eq!(
            rules("theorem t : True := by native_decide", check_no_native_decide).len(),
            1
        );
        assert!(rules("theorem t : True := by decide", check_no_native_decide).is_empty());
    }

    #[test]
    fn detects_partial_def() {
        assert_eq!(rules("partial def f : Nat := f", check_no_partial_def).len(), 1);
        assert!(rules("def f : Nat := 0", check_no_partial_def).is_empty());
    }

    #[test]
    fn detects_unsafe_modifier() {
        assert_eq!(rules("unsafe def f : Nat := 0", check_no_unsafe).len(), 1);
        // `partial` alone must not trip the `unsafe` rule.
        assert!(rules("partial def f : Nat := 0", check_no_unsafe).is_empty());
    }

    #[test]
    fn detects_dbg_trace() {
        assert_eq!(
            rules("def f : IO Unit := do\n  dbg_trace \"x\"", check_no_dbg_trace).len(),
            1
        );
        assert!(rules("def f : Nat := 0", check_no_dbg_trace).is_empty());
    }

    #[test]
    fn detects_debug_commands() {
        assert_eq!(rules("#eval 1 + 1", check_no_debug_command).len(), 1);
        assert_eq!(rules("#check Nat", check_no_debug_command).len(), 1);
        assert!(rules("def f : Nat := 0", check_no_debug_command).is_empty());
    }

    #[test]
    fn detects_unlimited_heartbeats() {
        assert_eq!(
            rules(
                "set_option maxHeartbeats 0 in\ntheorem t : True := trivial",
                check_no_unlimited_heartbeats
            )
            .len(),
            1
        );
        assert!(rules(
            "set_option maxHeartbeats 400000 in\ntheorem t : True := trivial",
            check_no_unlimited_heartbeats
        )
        .is_empty());
    }
}
