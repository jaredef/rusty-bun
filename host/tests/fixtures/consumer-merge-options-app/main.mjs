import mergeOptions from "merge-options";
const r = mergeOptions({ a: { b: 1 } }, { a: { c: 2 } });
process.stdout.write(JSON.stringify(r) + "\n");
