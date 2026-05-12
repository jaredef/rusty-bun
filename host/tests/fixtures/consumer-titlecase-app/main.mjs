// title-case ^4 — Title-case strings.
import { titleCase } from "title-case";

const lines = [];
lines.push("1 " + titleCase("hello world"));
lines.push("2 " + titleCase("the quick brown fox"));
lines.push("3 " + titleCase("a tale of two cities"));
lines.push("4 " + titleCase("over the rainbow"));
lines.push("5 " + titleCase("with a little help from my friends"));
lines.push("6 " + titleCase(""));

process.stdout.write(lines.join("\n") + "\n");
