import wa from "wrap-ansi";
const lines = [];
lines.push("1\n" + wa("Hello world this is a long line of text", 10));
lines.push("2\n" + wa("[31mred text wrapping[0m here", 12));
lines.push("3\n" + wa("nowrap", 50));
lines.push("4\n" + wa("a b c d e f g", 5, { hard: true }));
lines.push("5\n" + wa("verylongword", 5, { hard: true }));
lines.push("6\n" + wa("trim trailing spaces   ", 30, { trim: true }));
process.stdout.write(lines.join("\n") + "\n");
