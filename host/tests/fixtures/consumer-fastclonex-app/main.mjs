// fast-clone ^1 — fastest JSON-safe deep clone (used by Vue 1).
import clone from "fast-clone";

const lines = [];
const a = { x: 1, y: [2, { z: 3 }] };
const b = clone(a);
b.x = 99; b.y[1].z = 99;
lines.push("1 " + JSON.stringify(a));
lines.push("2 " + JSON.stringify(b));
lines.push("3 different=" + (a !== b));
lines.push("4 " + JSON.stringify(clone([1, 2, [3, [4, [5]]]])));
lines.push("5 " + JSON.stringify(clone({ a: { b: { c: { d: 1 } } } })));
lines.push("6 " + (clone(42) === 42));

process.stdout.write(lines.join("\n") + "\n");
