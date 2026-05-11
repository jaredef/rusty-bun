// ms ^2 — time string parser. Pure CJS. Tests the CJS-in-ESM bridge
// on a simple default-only export shape.
import ms from "ms";

const lines = [];

// 1: string → ms
lines.push("1 " + ms("2 days") + " " + ms("1h") + " " + ms("100ms") + " " + ms("-1s"));

// 2: ms → short string
lines.push("2 " + ms(60000) + " " + ms(2 * 24 * 3600 * 1000) + " " + ms(500));

// 3: ms → long string via options
lines.push("3 " + ms(60000, { long: true }) + " " + ms(2 * 24 * 3600 * 1000, { long: true }));

// 4: invalid input throws
{
  try { ms({}); lines.push("4 NOT_THROWN"); }
  catch (e) { lines.push("4 threw=" + (e instanceof Error)); }
}

// 5: zero + edge cases
lines.push("5 " + ms("0") + " " + ms(0));

process.stdout.write(lines.join("\n") + "\n");
