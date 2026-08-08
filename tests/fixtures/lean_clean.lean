/-!
Idiomatic Lean 4 that the grammar must read end to end.
-/

namespace Clean

/-- Classify a natural number by size. -/
def classify (n : Nat) : String :=
  if n == 0 then
    "zero"
  else if n < 10 then
    "small"
  else
    "large"

def sumTo : Nat → Nat
  | 0 => 0
  | n + 1 => n + 1 + sumTo n

theorem sumTo_zero : sumTo 0 = 0 := rfl

theorem add_zero_eq (n : Nat) : n + 0 = n := by
  simp

structure Point where
  x : Nat
  y : Nat
  deriving Repr

def Point.shift (p : Point) (dx : Nat) : Point :=
  { p with x := p.x + dx }

instance : ToString Point where
  toString p := s!"({p.x}, {p.y})"

inductive Shape where
  | circle (r : Nat)
  | rect (w h : Nat)

def Shape.area : Shape → Nat
  | .circle r => 3 * r * r
  | .rect w h => w * h

def describe (s : Shape) : String :=
  match s with
  | .circle _ => "circle"
  | .rect w h => if w == h then "square" else "rectangle"

end Clean
