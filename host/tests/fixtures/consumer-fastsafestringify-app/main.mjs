// fast-safe-stringify ^2 — JSON.stringify with cycle handling (used by pino).
import stringify from "fast-safe-stringify";

const lines = [];

const cyc = { a: 1 };
cyc.self = cyc;
lines.push("1 " + stringify(cyc));

const obj = { x: 1, y: { z: 2 } };
lines.push("2 " + stringify(obj));

const arr = [1, 2, 3];
arr.push(arr);
lines.push("3 " + stringify(arr));

lines.push("4 " + stringify({ a: undefined, b: null, c: NaN, d: Infinity }));

const big = { a: { b: { c: { d: { e: 1 } } } } };
lines.push("5 " + stringify(big));

const deepCyc = { a: { b: {} } };
deepCyc.a.b.parent = deepCyc;
lines.push("6 " + stringify(deepCyc));

process.stdout.write(lines.join("\n") + "\n");
