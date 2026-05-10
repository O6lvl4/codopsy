-- TODO: add type signatures
module Sample where

import Debug.Trace (trace)

process :: [Int] -> [Int]
process items = trace "debug" $ filter (> 10) items

unsafeStuff :: IO Int
unsafeStuff = do
  let x = head []
  let y = fromJust Nothing
  error "not implemented"
  return 0

empty :: () -> ()
empty _ = ()
