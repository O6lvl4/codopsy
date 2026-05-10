// Realistic Gleam code with various issues

// TODO: implement properly

pub fn process(data: List(Int)) -> List(Int) {
  todo
}

pub fn risky(x: Int) -> Int {
  case x {
    0 -> panic
    n -> n * 2
  }
}

pub fn assert_value(x: Result(Int, String)) -> Int {
  let assert Ok(val) = x
  val
}
