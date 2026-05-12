// qs ^6 — querystring with nested objects/arrays. Distinct from node:
// querystring (this is the deep-nesting variant used by express/axios).
import qs from "qs";

const lines = [];

lines.push("1 " + qs.stringify({ a: 1, b: "two", c: true }));
lines.push("2 " + qs.stringify({ a: { b: { c: "d" } } }));
lines.push("3 " + qs.stringify({ arr: [1, 2, 3] }));
lines.push("4 " + JSON.stringify(qs.parse("foo=bar&baz=qux")));
lines.push("5 " + JSON.stringify(qs.parse("a[b][c]=d")));
lines.push("6 " + JSON.stringify(qs.parse("arr[0]=x&arr[1]=y")));

process.stdout.write(lines.join("\n") + "\n");
