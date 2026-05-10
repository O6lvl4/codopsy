// expect: no-var, eqeqeq, no-eval, no-debugger
// expect: no-constant-condition, no-self-compare, use-isnan
// expect: todo-comment

// TODO: fix this
var x = 1;
if (x == 2) {}
eval("code");
debugger;
if (true) {}
if (x === x) {}
if (x === NaN) {}
