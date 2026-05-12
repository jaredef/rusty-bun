// parse-duration ^2 — duration string → ms (distinct from ms package).
import parse from "parse-duration";

const lines = [];
lines.push("1 " + parse("2h"));
lines.push("2 " + parse("1 day 3 hours"));
lines.push("3 " + parse("500ms"));
lines.push("4 " + parse("1.5h"));
lines.push("5 " + parse("1h30m"));
lines.push("6 " + parse("100", "s"));

process.stdout.write(lines.join("\n") + "\n");
