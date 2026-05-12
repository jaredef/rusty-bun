// escape-string-regexp ^5 — escape regex metacharacters in strings.
import escape from "escape-string-regexp";

const lines = [];
lines.push("1 " + escape("a.b"));
lines.push("2 " + escape("hello world"));
lines.push("3 " + escape("$10.00 + tax"));
lines.push("4 " + escape("a*b+c?d|e"));
lines.push("5 " + escape("[a]{1,2}"));
lines.push("6 " + new RegExp(escape("a.b")).test("a.b") + "/" + new RegExp(escape("a.b")).test("axb"));

process.stdout.write(lines.join("\n") + "\n");
