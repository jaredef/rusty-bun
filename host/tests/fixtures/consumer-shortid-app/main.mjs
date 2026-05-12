import shortid from "shortid";

const ids = Array.from({ length: 20 }, () => shortid.generate());
const allStrings = ids.every(s => typeof s === "string");
const allUnique = new Set(ids).size === ids.length;
const lengths = [...new Set(ids.map(s => s.length))].sort();
const charset = [...new Set(ids.join(""))].sort().join("");
const isValid = shortid.isValid(ids[0]);
const isValidGarbage = shortid.isValid("!!!@@@###");

process.stdout.write(JSON.stringify({
  count: ids.length,
  allStrings,
  allUnique,
  inRange: lengths.every(n => n >= 7 && n <= 14),
  charsetSizeInRange: charset.length >= 8 && charset.length <= 64,
  isValid,
  isValidGarbage,
}) + "\n");
