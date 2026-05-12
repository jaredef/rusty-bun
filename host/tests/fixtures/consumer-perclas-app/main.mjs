// classnames ^2 — conditional className utility (React-world standard).
import classNames from "classnames";

const lines = [];
lines.push("1 " + classNames("a", "b", "c"));
lines.push("2 " + classNames({ active: true, disabled: false }));
lines.push("3 " + classNames("base", { hover: true, focus: false }, "extra"));
lines.push("4 " + classNames(["foo", "bar", { baz: true }]));
lines.push("5 " + classNames(null, undefined, 0, "", "valid", false));
lines.push("6 " + classNames("a", { b: 1, c: "" }, ["d", { e: true, f: 0 }]));

process.stdout.write(lines.join("\n") + "\n");
