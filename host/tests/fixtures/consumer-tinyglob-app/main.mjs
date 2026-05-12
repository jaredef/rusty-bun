// micromatch ^4 — fast glob matcher (used by chokidar/etc).
import micromatch from "micromatch";

const lines = [];
lines.push("1 " + JSON.stringify(micromatch(["a.js", "b.ts", "c.js"], "*.js")));
lines.push("2 " + micromatch.isMatch("src/foo.js", "**/*.js"));
lines.push("3 " + JSON.stringify(micromatch(["a.js", "b.test.js", "c.js"], ["*.js", "!*.test.*"])));
lines.push("4 " + JSON.stringify(micromatch(["foo.txt", "bar.md", "baz.txt"], "*.{txt,md}")));
lines.push("5 " + micromatch.contains("a/b/c", "b"));
lines.push("6 " + micromatch.makeRe("*.js").source);

process.stdout.write(lines.join("\n") + "\n");
