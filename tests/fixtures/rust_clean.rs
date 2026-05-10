fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str) -> String {
    if name.is_empty() {
        "world".to_string()
    } else {
        name.to_string()
    }
}

fn main() {
    let result = add(1, 2);
    let greeting = greet("");
    let _ = (result, greeting);
}
