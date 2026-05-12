import pThrottle from "p-throttle";

const throttle = pThrottle({ limit: 2, interval: 50 });
const calls = [];
const fn = throttle(async (i) => { calls.push(i); return i * 2; });

const results = await Promise.all([0, 1, 2, 3, 4].map(fn));
process.stdout.write(JSON.stringify({
  results,
  callsLen: calls.length,
  callsSorted: [...calls].sort((a, b) => a - b),
}) + "\n");
