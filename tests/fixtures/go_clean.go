package main

func add(a, b int) int {
	return a + b
}

func greet(name string) string {
	if name == "" {
		return "world"
	}
	return name
}

func main() {
	result := add(1, 2)
	msg := greet("hello")
	println(result, msg)
}
