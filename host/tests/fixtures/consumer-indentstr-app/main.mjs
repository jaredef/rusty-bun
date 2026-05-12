// indent-string ^5 — prepend indent to each line.
import indentString from "indent-string";

const lines = [];
lines.push("1 " + JSON.stringify(indentString("a\nb\nc", 2)));
lines.push("2 " + JSON.stringify(indentString("hello", 4)));
lines.push("3 " + JSON.stringify(indentString("a\nb", 2, { indent: "-" })));
lines.push("4 " + JSON.stringify(indentString("", 5)));
lines.push("5 " + JSON.stringify(indentString("a\n\nb", 2)));
lines.push("6 " + JSON.stringify(indentString("x", 0)));

process.stdout.write(lines.join("\n") + "\n");
