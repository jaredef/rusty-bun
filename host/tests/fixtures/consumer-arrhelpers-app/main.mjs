// array-back ^6 — normalize value to array (similar to arrify but
// preserves iterables differently).
import arrayBack from "array-back";

const lines = [];
lines.push("1 " + JSON.stringify(arrayBack("hi")));
lines.push("2 " + JSON.stringify(arrayBack([1, 2, 3])));
lines.push("3 " + JSON.stringify(arrayBack(null)));
lines.push("4 " + JSON.stringify(arrayBack(undefined)));
lines.push("5 " + JSON.stringify(arrayBack({ a: 1 })));
lines.push("6 " + JSON.stringify(arrayBack(new Set([1, 2, 3]))));

process.stdout.write(lines.join("\n") + "\n");
