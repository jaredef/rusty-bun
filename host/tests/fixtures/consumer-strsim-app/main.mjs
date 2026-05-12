// string-similarity ^4 — Dice coefficient string similarity.
import stringSimilarity from "string-similarity";

const lines = [];
lines.push("1 " + stringSimilarity.compareTwoStrings("hello", "hello").toFixed(4));
lines.push("2 " + stringSimilarity.compareTwoStrings("kitten", "sitting").toFixed(4));
lines.push("3 " + stringSimilarity.compareTwoStrings("abc", "xyz").toFixed(4));
const r = stringSimilarity.findBestMatch("apple", ["apply", "appletree", "banana", "appl"]);
lines.push("4 best=" + r.bestMatch.target + " idx=" + r.bestMatchIndex);
lines.push("5 ratings=" + r.ratings.length);
lines.push("6 perfect=" + stringSimilarity.compareTwoStrings("a b", "a b").toFixed(4));

process.stdout.write(lines.join("\n") + "\n");
