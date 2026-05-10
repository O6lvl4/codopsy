#!/bin/bash
# TODO: add error handling

process() {
    for item in "$@"; do
        echo "$item"
        if [ "$item" -gt 10 ]; then
            if [ "$item" -gt 20 ]; then
                echo "big: $item"
            fi
        fi
    done
}

empty_func() {
    :
}

process 1 2 3 15 25
