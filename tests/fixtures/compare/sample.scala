// Realistic Scala code with various issues
// TODO: refactor

object Sample {
  def process(items: List[Int]): List[Int] = {
    items.filter(_ > 10)
  }

  def empty(): Unit = {}

  def complex(a: Int, b: Int, c: Int, d: Int, e: Int): Int = {
    a + b + c + d + e
  }
}
