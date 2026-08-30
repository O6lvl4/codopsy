package main

// Go 1.27: generic method (type parameters on a method declaration).
// This fixture guards that codopsy's tree-sitter-go can read it; if the
// grammar ever reverts, this file parses as one big error region and the
// test below sees it as `unanalyzed`.

type Stack[T any] struct {
	items []T
}

func (s *Stack[T]) Map[U any](f func(T) U) *Stack[U] {
	out := &Stack[U]{}
	for _, v := range s.items {
		out.items = append(out.items, f(v))
	}
	return out
}
