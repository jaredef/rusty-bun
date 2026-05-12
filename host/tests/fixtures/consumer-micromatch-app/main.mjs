import mm from "micromatch";

const out = {
  // Basic glob match
  starBasic: mm.isMatch("foo.js", "*.js"),
  notMatch: mm.isMatch("foo.ts", "*.js"),
  // Globstar
  starStar: mm.isMatch("src/a/b/c.js", "src/**/*.js"),
  // Negation
  negation: mm(["a.js", "b.js", "c.ts"], ["*.js", "!b.js"]).sort(),
  // Brace expansion
  brace: mm(["a.js", "b.js", "c.ts"], "*.{js,ts}").sort(),
  // Character class
  charClass: mm(["a.js", "b.js", "c.js"], "[ab].js").sort(),
  // Extglob
  extglob: mm.isMatch("foo.js", "@(*.js|*.ts)"),
  // Multiple patterns
  multi: mm(["src/a.js", "test/b.js", "build/c.js"], ["src/**", "test/**"]).sort(),
  // Capture groups
  capture: mm.capture("src/*.js", "src/foo.js"),
};

process.stdout.write(JSON.stringify(out) + "\n");
