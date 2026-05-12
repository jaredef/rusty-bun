import { hash, digest, isEqual, serialize } from "ohash";
const lines = [];
lines.push("1 " + hash({ a: 1, b: 2 }));
lines.push("2 same=" + (hash({ a: 1, b: 2 }) === hash({ b: 2, a: 1 })));
lines.push("3 diff=" + (hash({ a: 1 }) !== hash({ a: 2 })));
lines.push("4 " + hash([1, 2, 3]));
lines.push("5 " + hash("hello"));
lines.push("6 eq=" + isEqual({a:1,b:2}, {b:2,a:1}));
process.stdout.write(lines.join("\n") + "\n");
