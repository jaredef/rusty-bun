import deepmerge from "deepmerge";
const r = deepmerge({ a: { b: 1 } }, { a: { c: 2 } });
process.stdout.write(JSON.stringify(r) + "\n");
