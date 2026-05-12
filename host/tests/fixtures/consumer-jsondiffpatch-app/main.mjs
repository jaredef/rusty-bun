// jsondiffpatch ^0.6 — structural JSON diff/patch with object-hash for
// array reconciliation. Distinct axis from flatted/deepmerge.
import * as jdp from "jsondiffpatch";

const lines = [];
const inst = jdp.create({ objectHash: o => o && o.id });

const a = { x: 1, y: 2, z: { w: 3 } };
const b = { x: 1, y: 20, z: { w: 3, q: 4 } };
const d1 = inst.diff(a, b);
lines.push("1 " + JSON.stringify(d1));

const a2 = [{ id: 1, v: "a" }, { id: 2, v: "b" }];
const b2 = [{ id: 2, v: "b2" }, { id: 1, v: "a" }, { id: 3, v: "c" }];
const d2 = inst.diff(a2, b2);
lines.push("2 hasDelta=" + (d2 !== undefined));

const a3 = { n: 5 };
const b3 = { n: 10 };
const d3 = inst.diff(a3, b3);
const patched = inst.patch(JSON.parse(JSON.stringify(a3)), d3);
lines.push("3 " + JSON.stringify(patched));

const unpatched = inst.unpatch(JSON.parse(JSON.stringify(patched)), d3);
lines.push("4 " + JSON.stringify(unpatched));

const rev = inst.reverse(d3);
lines.push("5 hasReverse=" + (rev !== undefined));

const same = inst.diff({ a: 1 }, { a: 1 });
lines.push("6 sameDiff=" + (same === undefined));

process.stdout.write(lines.join("\n") + "\n");
