"""A clean Python module with no lint violations."""


def add(a: int, b: int) -> int:
    return a + b


def greet(name: str) -> str:
    if not name:
        return "world"
    return name
