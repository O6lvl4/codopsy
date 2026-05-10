# Realistic Python code with various issues for linter comparison
from os import *
import sys

DEBUG = True

def process(items, cache={}):
    global DEBUG
    for item in items:
        print(f"processing {item}")
        if item > 100:
            if item > 200:
                pass

    try:
        result = sum(items)
    except:
        pass

    try:
        x = 1 / 0
    except ValueError:
        raise

    eval("print('hello')")

    assert len(items) > 0

    return result

class Broken:
    def __init__(self):
        return 42
