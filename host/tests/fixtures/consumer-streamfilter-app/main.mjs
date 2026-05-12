// filter-obj ^6 — filter object keys by predicate.
import { includeKeys, excludeKeys } from "filter-obj";

const lines = [];
const obj = { a: 1, b: 2, c: 3, d: 4 };

lines.push("1 " + JSON.stringify(includeKeys(obj, (k, v) => v > 2)));
lines.push("2 " + JSON.stringify(includeKeys(obj, k => k === "a" || k === "c")));
lines.push("3 " + JSON.stringify(includeKeys(obj, ["a", "d"])));
lines.push("4 " + JSON.stringify(excludeKeys(obj, ["b", "c"])));
lines.push("5 " + JSON.stringify(includeKeys(obj, k => k > "b")));
lines.push("6 empty=" + JSON.stringify(includeKeys({}, () => true)));

process.stdout.write(lines.join("\n") + "\n");
