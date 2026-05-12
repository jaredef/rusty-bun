// minimatch ^10 — glob → regex matching (used by npm, glob, etc).
import { minimatch } from "minimatch";

const lines = [];
lines.push("1 " + minimatch("foo.js", "*.js"));
lines.push("2 " + minimatch("src/foo.js", "**/*.js"));
lines.push("3 " + minimatch("src/foo.js", "src/*.js"));
lines.push("4 " + minimatch("src/nested/foo.js", "src/*.js"));
lines.push("5 " + minimatch("foo.test.js", "*.{js,ts}"));
lines.push("6 " + minimatch("foo", "f?o"));

process.stdout.write(lines.join("\n") + "\n");
