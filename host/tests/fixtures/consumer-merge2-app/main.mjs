// merge-deep ^3 — jonschlinkert-family deep merge (mutates first arg).
import mergeDeep from "merge-deep";

const lines = [];
lines.push("1 " + JSON.stringify(mergeDeep({ a: 1 }, { b: 2 })));
lines.push("2 " + JSON.stringify(mergeDeep({ a: { x: 1 } }, { a: { y: 2 } })));
lines.push("3 " + JSON.stringify(mergeDeep({ a: 1, b: 2 }, { a: 99 })));
lines.push("4 " + JSON.stringify(mergeDeep({}, { a: [1, 2] }, { a: [3, 4] })));
lines.push("5 " + JSON.stringify(mergeDeep({ a: { b: { c: 1 } } }, { a: { b: { d: 2 } } })));
lines.push("6 " + JSON.stringify(mergeDeep({ a: null }, { a: { b: 1 } })));

process.stdout.write(lines.join("\n") + "\n");
