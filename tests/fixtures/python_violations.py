# expect: no-bare-except, no-print, no-eval, no-mutable-default
# expect: no-global, no-star-import, no-return-in-init
# expect: todo-comment

# TODO: cleanup
from os import *

print("hello")
eval("1+1")

def bad_default(x=[]):
    pass

def use_global():
    global y
    y = 1

try:
    pass
except:
    pass

class Foo:
    def __init__(self):
        return 42
