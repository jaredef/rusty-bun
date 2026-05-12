// clsx ^2 — smaller faster alt to classnames (preact/Material-UI default).
import clsx from "clsx";

const lines = [];
lines.push("1 " + clsx("a", "b", "c"));
lines.push("2 " + clsx({ on: true, off: false }));
lines.push("3 " + clsx("base", { hover: true }, ["extra", "more"]));
lines.push("4 " + clsx(null, "valid", undefined, "", 0, "yes"));
lines.push("5 " + clsx("a", ["b", ["c", { d: true }]]));
lines.push("6 " + clsx());

process.stdout.write(lines.join("\n") + "\n");
