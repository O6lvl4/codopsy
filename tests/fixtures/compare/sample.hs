-- Realistic Haskell code with various issues
-- TODO: add type signatures

module Sample where

process :: [Int] -> [Int]
process items = filter (> 10) items

empty :: () -> ()
empty _ = ()

complex :: Int -> Int -> Int -> Int -> Int -> Int
complex a b c d e = a + b + c + d + e
