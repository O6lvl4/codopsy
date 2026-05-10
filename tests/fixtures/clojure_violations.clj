; expect: no-println, no-def-in-def
; expect: todo-comment

; TODO: fix
(ns violations)

(println "hello")

(defn outer []
  (defn inner [] 42)
  (inner))
