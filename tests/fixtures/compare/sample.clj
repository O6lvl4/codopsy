;; Realistic Clojure code with various issues
(ns sample.core)

;; TODO: clean up debug output
(println "loading module")

(defn process [items]
  (println "processing" (count items))
  (defn helper [x]
    (Thread/sleep 100)
    (* x 2))
  (map helper items))

(defn with-reflection [obj]
  (.getClass obj))

(defn empty-fn [])
