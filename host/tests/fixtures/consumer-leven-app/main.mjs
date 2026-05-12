// leven ^4 — Levenshtein distance (distinct algorithm from Dice).
import leven from "leven";

const lines = [];
lines.push("1 " + leven("kitten", "sitting"));
lines.push("2 " + leven("", "abc"));
lines.push("3 " + leven("abc", ""));
lines.push("4 " + leven("same", "same"));
lines.push("5 " + leven("flaw", "lawn"));
lines.push("6 " + leven("intention", "execution"));

process.stdout.write(lines.join("\n") + "\n");
