// Realistic Swift code with various issues
// TODO: refactor

import Foundation

func process(_ items: [Int]) -> [Int] {
    var result: [Int] = []
    for item in items {
        if item > 10 {
            if item > 20 {
                result.append(item)
            }
        }
    }
    return result
}

func empty() {}

func complex(_ a: Int, _ b: Int, _ c: Int, _ d: Int, _ e: Int) -> Int {
    return a + b + c + d + e
}
