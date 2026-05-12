// is-plain-obj ^4 — strict plain-object check.
import isPlainObj from "is-plain-obj";

const lines = [];
lines.push("1 " + isPlainObj({}) + "/" + isPlainObj({ a: 1 }));
lines.push("2 " + isPlainObj(null) + "/" + isPlainObj(undefined));
lines.push("3 " + isPlainObj([]) + "/" + isPlainObj(new Date()));
lines.push("4 " + isPlainObj("x") + "/" + isPlainObj(42));
lines.push("5 " + isPlainObj(new Map()) + "/" + isPlainObj(/r/));
lines.push("6 " + isPlainObj(Object.create(null)));

process.stdout.write(lines.join("\n") + "\n");
