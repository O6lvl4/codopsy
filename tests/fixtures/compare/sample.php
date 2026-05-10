<?php
// TODO: add validation

function process($items) {
    var_dump($items);
    dd($items);
    eval('$x = 1;');

    if ($items == null) {
        die("no items");
    }

    @file_get_contents("url");

    foreach ($items as $item) {
        echo $item . "\n";
    }
}

function empty_func() {}

function complex($a, $b, $c, $d, $e) {
    return $a + $b + $c + $d + $e;
}

process([1, 2, 3]);
