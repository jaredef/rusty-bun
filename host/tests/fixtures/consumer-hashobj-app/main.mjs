// object-hash ^3 — stable deterministic hash for arbitrary JS values.
import hash from "object-hash";

const lines = [];
lines.push("1 " + hash({ a: 1, b: 2 }));
lines.push("2 sameAcrossKeyOrder=" + (hash({ a: 1, b: 2 }) === hash({ b: 2, a: 1 })));
lines.push("3 diff=" + (hash({ a: 1 }) !== hash({ a: 2 })));
lines.push("4 " + hash([1, 2, 3]));
lines.push("5 " + hash("hello"));
lines.push("6 " + hash({ nested: { deep: [1, { x: "y" }] } }));

process.stdout.write(lines.join("\n") + "\n");
