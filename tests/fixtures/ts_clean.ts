// 型の位置でしか使われない import も「使っている」と数える。
// identifier だけを見ていた頃は、この4つすべてが unused-import になった。
import type { Alpha } from "./alpha";
import { Bravo } from "./bravo";
import { Charlie } from "./charlie";
import { Delta } from "./delta";

export function takesType(x: Alpha): number {
  return x.n;
}

export function returnsType(): Bravo {
  return { n: 1 };
}

export class Impl implements Charlie {
  n = 1;
}

export default { Delta };
