// TODO: refactor
import Foundation

func process(_ items: [Int]) -> [Int] {
    print("processing")
    NSLog("debug info")

    let value: Int? = items.first
    let forced = value!

    let result = try! riskyOperation()
    let casted = value as! Double

    if forced > 10 {
        fatalError("too big")
    }

    return items.filter { $0 > forced }
}

func riskyOperation() throws -> Int { return 42 }

func empty() {}
