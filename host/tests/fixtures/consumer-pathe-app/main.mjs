// pathe ^2 — universal path utilities (cross-platform forward slashes).
import { resolve, join, normalize, dirname, basename, extname, parse } from "pathe";

const lines = [];
lines.push("1 " + join("a", "b", "c"));
lines.push("2 " + normalize("/a/b/../c/./d"));
lines.push("3 " + dirname("/foo/bar/baz.txt"));
lines.push("4 " + basename("/foo/bar/baz.txt"));
lines.push("5 " + extname("/foo/bar.tar.gz"));
lines.push("6 " + JSON.stringify(parse("/a/b/c.ext")));

process.stdout.write(lines.join("\n") + "\n");
