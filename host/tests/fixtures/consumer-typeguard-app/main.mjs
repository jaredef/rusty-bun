// kind-of ^6 — reliable typeof replacement (handles wrappers, etc).
import kindOf from "kind-of";

const lines = [];
lines.push("1 " + kindOf(42) + "/" + kindOf("x") + "/" + kindOf(true));
lines.push("2 " + kindOf(null) + "/" + kindOf(undefined));
lines.push("3 " + kindOf([1, 2, 3]) + "/" + kindOf({}));
lines.push("4 " + kindOf(new Date()) + "/" + kindOf(/x/));
lines.push("5 " + kindOf(new Set()) + "/" + kindOf(new Map()));
lines.push("6 " + kindOf(Buffer.from("hi")) + "/" + kindOf(function () {}));

process.stdout.write(lines.join("\n") + "\n");
