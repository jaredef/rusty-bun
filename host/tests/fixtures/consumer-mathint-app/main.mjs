// string-natural-compare ^3 — Compare strings naturally (good for filenames).
import naturalCompare from "string-natural-compare";

const lines = [];
lines.push("1 " + naturalCompare("a2", "a10"));
lines.push("2 " + naturalCompare("file1", "file2"));
lines.push("3 " + naturalCompare("zzz", "aaa"));
lines.push("4 " + naturalCompare("v1.10", "v1.9"));
lines.push("5 " + JSON.stringify(["a10", "a1", "a2", "a11"].sort(naturalCompare)));
lines.push("6 " + naturalCompare("eq", "eq"));

process.stdout.write(lines.join("\n") + "\n");
