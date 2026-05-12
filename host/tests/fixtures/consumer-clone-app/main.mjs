// clone ^2 — deep clone (distinct from rfdc/structuredClone tests).
import clone from "clone";

const lines = [];

const a = { x: 1, y: [2, { z: 3 }], d: new Date(0) };
const b = clone(a);
b.x = 99; b.y[1].z = 99;
lines.push("1 " + JSON.stringify(a));

const c = { arr: [1, 2, 3] };
c.self = c;
const d = clone(c);
lines.push("2 cycleOk=" + (d.self === d) + " arr=" + JSON.stringify(d.arr));

const e = clone(/abc/gi);
lines.push("3 " + (e instanceof RegExp) + " src=" + e.source + " flags=" + e.flags);

const f = clone(new Map([["a", 1], ["b", 2]]));
lines.push("4 mapSize=" + f.size + " a=" + f.get("a"));

const g = clone({ nested: { deep: { v: [1, 2, 3] } } });
g.nested.deep.v.push(4);
lines.push("5 deep=" + g.nested.deep.v.join(","));

lines.push("6 prim=" + clone(42) + " " + clone("hi"));

process.stdout.write(lines.join("\n") + "\n");
