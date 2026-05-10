// expect: no-unwrap, no-dbg, no-todo, no-println, no-empty-function
// expect: needless-return, bool-comparison, eq-op
// expect: todo-comment

// TODO: refactor
fn main() {
    let x = Some(1).unwrap();
    dbg!(x);
    println!("hello");
    todo!();
}

fn empty() {}

fn needless() -> i32 {
    return 42;
}

fn bool_cmp(x: bool) -> bool {
    x == true
}

fn self_cmp(a: i32) -> bool {
    a == a
}
