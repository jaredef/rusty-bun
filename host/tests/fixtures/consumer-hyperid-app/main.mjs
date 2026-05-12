// hyperid ^4 — fast unique ID generator (UUID-based prefix + monotonic
// counter suffix). Distinct from nanoid/ulid/uuid (different structure).
import hyperid from "hyperid";

const lines = [];

const inst = hyperid({ urlSafe: true });
const ids = Array.from({ length: 5 }, () => inst());

// Check all unique
const uniq = new Set(ids);
lines.push("1 uniqueCount=" + uniq.size);

// Each is a non-empty string
lines.push("2 allStr=" + ids.every(s => typeof s === "string" && s.length > 0));

// urlSafe → no slashes, plus, or equals
lines.push("3 urlSafe=" + ids.every(s => !/[/+=]/.test(s)));

// All same length (fixed-length ids)
const lens = new Set(ids.map(s => s.length));
lines.push("4 oneLen=" + (lens.size === 1));

// Strict prefix matches for first N-suffix chars
const pfx = (s) => s.slice(0, 22);
const samePrefix = ids.every(s => pfx(s) === pfx(ids[0]));
lines.push("5 samePrefix=" + samePrefix);

// Fresh instance has different prefix
const inst2 = hyperid({ urlSafe: true });
const id2 = inst2();
lines.push("6 diffPrefix=" + (pfx(id2) !== pfx(ids[0])));

process.stdout.write(lines.join("\n") + "\n");
