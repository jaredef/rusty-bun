// just-* ^2 — a family of single-purpose mini utilities (no deps each).
import extend from "just-extend";
import clone from "just-clone";
import pick from "just-pick";
import omit from "just-omit";

const lines = [];
lines.push("1 " + JSON.stringify(extend({}, { a: 1 }, { b: 2 }, { c: 3 })));
lines.push("2 " + JSON.stringify(extend(true, {}, { a: { b: 1 } }, { a: { c: 2 } })));
lines.push("3 " + JSON.stringify(clone({ x: 1, y: [2, 3] })));
lines.push("4 " + JSON.stringify(pick({ a: 1, b: 2, c: 3 }, ["a", "c"])));
lines.push("5 " + JSON.stringify(omit({ a: 1, b: 2, c: 3 }, ["b"])));
lines.push("6 isolated=" + (clone({ a: [1, 2] }).a !== ({ a: [1, 2] }).a));

process.stdout.write(lines.join("\n") + "\n");
