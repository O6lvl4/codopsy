-- expect: no-sorry, no-axiom, no-native-decide, no-unsafe, no-unlimited-heartbeats
-- expect: no-partial-def, no-dbg-trace, no-debug-command, todo-comment

namespace Violations

-- TODO: finish this proof
theorem unfinished (a b : Nat) : a + b = b + a := sorry

axiom untrusted : ∀ (α : Type), Nonempty α

theorem byNative : True := by
  native_decide

unsafe def raw (n : Nat) : Nat := n

set_option maxHeartbeats 0 in
theorem slow : True := trivial

partial def spin (n : Nat) : Nat := spin n

def trace (n : Nat) : Nat :=
  dbg_trace "n"
  n

#eval raw 1

end Violations
