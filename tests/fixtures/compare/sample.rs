// Realistic Rust code with various issues for linter comparison
use std::collections::HashMap;

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    // needless return
    let _total = compute(&data);

    // dbg and println
    dbg!(&data);
    println!("result: {}", _total);

    // unwrap
    let map: HashMap<&str, i32> = HashMap::new();
    let _val = map.get("key").unwrap();

    // todo
    todo!("implement this");
}

fn compute(data: &[i32]) -> i32 {
    return data.iter().sum();
}

fn empty_fn() {}

// bool comparison
fn is_valid(x: bool) -> bool {
    x == true
}

// eq-op
fn always_true(a: i32) -> bool {
    a == a
}

// needless bool
fn check(x: i32) -> bool {
    if x > 0 { true } else { false }
}
