// arrify ^3 — value → array converter.
import arrify from "arrify";

const lines = [];
lines.push("1 " + JSON.stringify(arrify("hi")));
lines.push("2 " + JSON.stringify(arrify([1, 2, 3])));
lines.push("3 " + JSON.stringify(arrify(null)));
lines.push("4 " + JSON.stringify(arrify(undefined)));
lines.push("5 " + JSON.stringify(arrify(42)));
lines.push("6 " + JSON.stringify(arrify(new Set([1, 2, 3]))));

process.stdout.write(lines.join("\n") + "\n");
