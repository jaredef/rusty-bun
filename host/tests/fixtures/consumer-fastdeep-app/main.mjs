// fast-deep-equal ^3 — faster deep-equal alt (used by ajv).
import equal from "fast-deep-equal";

const lines = [];
lines.push("1 " + equal({ a: 1 }, { a: 1 }));
lines.push("2 " + equal({ a: 1 }, { a: 2 }));
lines.push("3 " + equal([1, 2, 3], [1, 2, 3]));
lines.push("4 " + equal({ a: { b: { c: 1 } } }, { a: { b: { c: 1 } } }));
lines.push("5 " + equal(new Date(0), new Date(0)));
lines.push("6 " + equal(/foo/g, /foo/g));

process.stdout.write(lines.join("\n") + "\n");
