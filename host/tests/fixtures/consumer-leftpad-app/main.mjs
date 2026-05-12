// left-pad ^1 — the historic CommonJS string-padding lib. CJS-in-ESM
// path exercise.
import leftPad from "left-pad";

const lines = [];
lines.push("1 [" + leftPad("x", 5) + "]");
lines.push("2 [" + leftPad("x", 5, "0") + "]");
lines.push("3 [" + leftPad(42, 6) + "]");
lines.push("4 [" + leftPad("xxx", 2) + "]");
lines.push("5 [" + leftPad("", 4, "-") + "]");
lines.push("6 [" + leftPad("ok", 7, ".") + "]");

process.stdout.write(lines.join("\n") + "\n");
