// expect: no-todo, no-panic
// expect: todo-comment

// TODO: implement
pub fn unfinished() -> Int {
  todo
}

pub fn crash() -> Int {
  panic
}
