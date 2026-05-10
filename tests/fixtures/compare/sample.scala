// TODO: refactor
object Sample {
  def process(items: List[Int]): List[Int] = {
    println("processing")
    var count = 0
    items.foreach { item =>
      count += 1
      val x: Any = item.asInstanceOf[Any]
    }
    if (items.isEmpty) return List()
    val n: String = null
    items.filter(_ > 10)
  }

  def empty(): Unit = {}
}
