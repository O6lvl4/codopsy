// Realistic JS code with various issues for linter comparison
var total = 0;
const data = [1, 2, 3];

function processItem(item) {
  if (typeof item === "nunber") {
    total += item;
  }
  if (item == null) {
    return;
  }
  if (true) {
    console.log("processing", item);
  }
  eval("total += " + item);
  debugger;
}

function emptyHandler() {}

function unreachable() {
  return total;
  console.log("never reached");
}

for (var i = 0; i < data.length; i++) {
  processItem(data[i]);
}

if (total === NaN) {
  console.log("bad");
}

const x = total;
x = x;
