// expect: no-any, no-console, no-empty-function, no-unreachable
// expect: todo-comment

// TODO: migrate
const x: any = 1;
console.log(x);
function empty(): void {}
function dead(): number {
  return 1;
  const y = 2;
}
