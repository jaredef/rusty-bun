// stringify-object ^6 — JS-syntax stringifier (vs JSON).
import stringifyObject from "stringify-object";

const lines = [];
lines.push("1 " + stringifyObject({ a: 1, b: "two" }));
lines.push("2 " + stringifyObject([1, 2, 3]));
lines.push("3 " + stringifyObject({ x: { y: 1 } }, { indent: "  " }));
lines.push("4 " + stringifyObject({ s: "with 'quotes'" }, { singleQuotes: true }));
lines.push("5 " + stringifyObject({ s: "double" }, { singleQuotes: false }));
lines.push("6 " + stringifyObject(new Set([1, 2, 3])));

process.stdout.write(lines.join("\n") + "\n");
