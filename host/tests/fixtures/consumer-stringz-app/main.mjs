// stringz ^2 — Unicode-aware string length/substring (CJK/emoji widths).
import { length, substring, limit, indexOf } from "stringz";

const lines = [];
lines.push("1 ascii=" + length("hello"));
lines.push("2 cjk=" + length("中文"));
lines.push("3 emoji=" + length("👨‍👩‍👧"));
lines.push("4 sub=" + substring("hello world", 0, 5));
lines.push("5 limit=" + limit("hello world", 5, "..."));
lines.push("6 idx=" + indexOf("hello", "ll"));

process.stdout.write(lines.join("\n") + "\n");
