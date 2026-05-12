// random-int ^3 — uniform random integer in range.
import randomInt from "random-int";

const lines = [];
let allInRange = true;
for (let i = 0; i < 100; i++) {
  const r = randomInt(1, 10);
  if (r < 1 || r > 10) { allInRange = false; break; }
}
lines.push("1 range1to10=" + allInRange);

let allInRangeNeg = true;
for (let i = 0; i < 50; i++) {
  const r = randomInt(-5, 5);
  if (r < -5 || r > 5) { allInRangeNeg = false; break; }
}
lines.push("2 neg=" + allInRangeNeg);

let allZero = true;
for (let i = 0; i < 10; i++) {
  if (randomInt(0, 0) !== 0) { allZero = false; break; }
}
lines.push("3 zeroOnly=" + allZero);

const single = randomInt(7); // single-arg = 0..7
lines.push("4 single7Type=" + (typeof single === "number" && single >= 0 && single <= 7));

const ints = new Set();
for (let i = 0; i < 100; i++) ints.add(randomInt(1, 100));
lines.push("5 spreadCount=" + (ints.size > 30));

lines.push("6 isFn=" + (typeof randomInt === "function"));

process.stdout.write(lines.join("\n") + "\n");
