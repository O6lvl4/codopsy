package main

import (
	"fmt"
	"os"
)

func main() {
	fmt.Println("hello")

	panic("fatal error")

	for i := 0; i < 10; i++ {
		defer fmt.Println(i)
	}

	os.Exit(1)
}

func empty() {}

func unreachable() int {
	return 42
	fmt.Println("never")
	return 0
}
