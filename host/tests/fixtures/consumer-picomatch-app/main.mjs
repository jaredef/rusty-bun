// picomatch ^4 — glob pattern matcher. Pure-CJS through bridge.
// Used by chokidar, micromatch, anymatch, fast-glob, jest, vitest, etc.
import picomatch from "picomatch";

const lines = [];

// 1: basic star
{
  const m = picomatch("*.js");
  lines.push("1 a.js=" + m("a.js") + " a.ts=" + m("a.ts") + " sub/a.js=" + m("sub/a.js"));
}

// 2: globstar
{
  const m = picomatch("**/*.test.ts");
  lines.push("2 " + m("a.test.ts") + " " + m("nest/a.test.ts") + " " + m("a.ts"));
}

// 3: brace expansion
{
  const m = picomatch("**/*.{js,ts}");
  lines.push("3 " + m("a.js") + " " + m("a.ts") + " " + m("a.tsx"));
}

// 4: negation
{
  const m = picomatch(["**/*.js", "!**/*.test.js"]);
  lines.push("4 src.js=" + m("src.js") + " src.test.js=" + m("src.test.js"));
}

// 5: character class
{
  const m = picomatch("file-[0-9].txt");
  lines.push("5 " + m("file-5.txt") + " " + m("file-a.txt") + " " + m("file-12.txt"));
}

// 6: scan (extract metadata)
{
  const s = picomatch.scan("src/**/*.js");
  lines.push("6 base=" + s.base + " glob=" + s.glob + " isGlob=" + s.isGlob);
}

// 7: makeRe
{
  const re = picomatch.makeRe("**/*.md");
  lines.push("7 isRegExp=" + (re instanceof RegExp) + " matches=" + re.test("docs/README.md"));
}

process.stdout.write(lines.join("\n") + "\n");
