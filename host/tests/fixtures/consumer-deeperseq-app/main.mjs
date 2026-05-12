// deep-equal ^2 — classic deep equality lib (transitive deps used in
// browserify/babel pipeline).
import equal from "deep-equal";

const lines = [];
lines.push("1 " + equal({ a: 1, b: 2 }, { b: 2, a: 1 }));
lines.push("2 " + equal([1, [2, [3]]], [1, [2, [3]]]));
lines.push("3 " + equal({ a: 1 }, { a: 2 }));
lines.push("4 " + equal(new Date(0), new Date(0)));
lines.push("5 " + equal({ a: NaN }, { a: NaN }, { strict: true }));
lines.push("6 " + equal({ a: undefined }, {}, { strict: true }));

process.stdout.write(lines.join("\n") + "\n");
