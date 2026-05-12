// string-argv ^0.3 — POSIX shell argv splitter.
import stringArgv from "string-argv";

const lines = [];
lines.push("1 " + JSON.stringify(stringArgv("a b c")));
lines.push("2 " + JSON.stringify(stringArgv('cmd "with spaces" -o out.txt')));
lines.push("3 " + JSON.stringify(stringArgv("a 'b c' d")));
lines.push("4 " + JSON.stringify(stringArgv("--flag value --other")));
lines.push("5 " + JSON.stringify(stringArgv("")));
lines.push("6 " + JSON.stringify(stringArgv("a\\ b c")));

process.stdout.write(lines.join("\n") + "\n");
