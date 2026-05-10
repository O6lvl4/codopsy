<?php
// Realistic PHP code with various issues
// TODO: add validation

function process($items) {
    foreach ($items as $item) {
        echo $item . "\n";
        if ($item > 10) {
            if ($item > 20) {
                echo "big: $item\n";
            }
        }
    }
}

function empty_func() {}

function complex($a, $b, $c, $d, $e) {
    return $a + $b + $c + $d + $e;
}

process([1, 2, 3, 15, 25]);
