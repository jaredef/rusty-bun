import { Map, List, Set, OrderedMap, fromJS } from "immutable";

const lines = [];

// 1: Map
{
  const m = Map({ a: 1, b: 2 });
  const m2 = m.set("c", 3);
  lines.push("1 origSize=" + m.size + " newSize=" + m2.size + " differ=" + (m !== m2));
}

// 2: List
{
  const l = List([1, 2, 3]);
  const l2 = l.push(4);
  lines.push("2 orig=" + l.toJS() + " new=" + l2.toJS());
}

// 3: Set
{
  const s = Set([1, 2, 3, 2, 1]);
  lines.push("3 size=" + s.size + " has2=" + s.has(2) + " values=" + JSON.stringify(s.toJS().sort()));
}

// 4: fromJS deep conversion
{
  const obj = { a: [1, 2], b: { c: 3 } };
  const i = fromJS(obj);
  lines.push("4 aType=" + i.get("a").constructor.name + " bType=" + i.get("b").constructor.name);
}

// 5: structural equality
{
  const a = Map({ x: 1, y: List([2, 3]) });
  const b = Map({ x: 1, y: List([2, 3]) });
  lines.push("5 equal=" + a.equals(b) + " is===" + (a === b));
}

// 6: chain
{
  const r = List([1, 2, 3, 4, 5])
    .filter(n => n % 2 === 1)
    .map(n => n * n)
    .reduce((acc, n) => acc + n, 0);
  lines.push("6 sumSqOdd=" + r);
}

process.stdout.write(lines.join("\n") + "\n");
