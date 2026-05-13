import ignore from "ignore";

const ig = ignore().add(["node_modules", "*.log", "build/", "!build/keep.txt"]);

process.stdout.write(JSON.stringify({
  ignoresNodeModules: ig.ignores("node_modules/foo"),
  ignoresLog: ig.ignores("debug.log"),
  ignoresBuild: ig.ignores("build/output.js"),
  ignoresBuildKeep: ig.ignores("build/keep.txt"),
  doesntIgnoreSrc: !ig.ignores("src/index.js"),
  filter: ig.filter(["src/x.js", "node_modules/y.js", "z.log"]).sort(),
}) + "\n");
