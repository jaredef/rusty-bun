// memoize-one ^6 — cache only the most recent invocation.
import memoizeOne from "memoize-one";

const lines = [];

let calls = 0;
const fn = memoizeOne((x, y) => { calls++; return x + y; });
fn(1, 2); fn(1, 2); fn(1, 2);
lines.push("1 calls=" + calls);

fn(3, 4);
lines.push("2 newArgs=" + calls);

fn(1, 2);
lines.push("3 backToOld=" + calls);

const compareEq = memoizeOne((a) => ({ doubled: a * 2 }), (newArgs, lastArgs) => true);
compareEq(1); compareEq(2); compareEq(3);
lines.push("4 alwaysCached=" + (compareEq(99).doubled === 2));

let calls5 = 0;
const fn5 = memoizeOne(async (x) => { calls5++; return x + 100; });
await fn5(1); await fn5(1);
lines.push("5 asyncCalls=" + calls5);

lines.push("6 isFn=" + (typeof memoizeOne === "function"));

process.stdout.write(lines.join("\n") + "\n");
