// Realistic TS code with various issues
const data: any = [1, 2, 3];
var count = 0;

function process(item: any): void {
  if (item == null) return;
  console.log("processing", item);
  count++;
}

function empty(): void {}

function unreachable(): number {
  return 42;
  const x = 1;
}

debugger;

if (true) {
  eval("alert(1)");
}

if (count === NaN) {}
