// strip-indent ^4 — strip common leading whitespace from each line.
import stripIndent from "strip-indent";

const lines = [];
lines.push("1 " + JSON.stringify(stripIndent("    hello\n    world")));
lines.push("2 " + JSON.stringify(stripIndent("  a\n    b\n  c")));
lines.push("3 " + JSON.stringify(stripIndent("noindent")));
lines.push("4 " + JSON.stringify(stripIndent("")));
lines.push("5 " + JSON.stringify(stripIndent("\thello\n\tworld")));
lines.push("6 " + JSON.stringify(stripIndent("    a\n\n    b")));

process.stdout.write(lines.join("\n") + "\n");
