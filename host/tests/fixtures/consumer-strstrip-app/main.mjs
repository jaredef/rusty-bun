// strip-ansi ^7 — remove ANSI escape codes from strings.
import stripAnsi from "strip-ansi";

const lines = [];
lines.push("1 " + stripAnsi("[31mred[0m"));
lines.push("2 " + stripAnsi("[1mbold[22m and [3mitalic[23m"));
lines.push("3 " + stripAnsi("no ansi"));
lines.push("4 " + stripAnsi("[38;5;82mfg256[0m"));
lines.push("5 " + stripAnsi("[1;31;42mmixed[0m"));
lines.push("6 " + stripAnsi("a[0mb[0mc"));

process.stdout.write(lines.join("\n") + "\n");
