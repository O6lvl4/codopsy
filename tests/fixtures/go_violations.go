package main

// expect: no-panic, no-fmt-print, no-defer-in-loop
// expect: todo-comment, no-empty-function

// TODO: fix this
import "fmt"

func main() {
	panic("error")
	fmt.Println("hello")
	for i := 0; i < 10; i++ {
		defer fmt.Println(i)
	}
}

func empty() {}
