import pSeries from "p-series";
const lines = [];

const order = [];
const r = await pSeries([
  async () => { order.push("a"); return 1; },
  async () => { order.push("b"); return 2; },
  async () => { order.push("c"); return 3; },
]);
lines.push("1 order=" + order.join(","));
lines.push("2 " + JSON.stringify(r));

const t1 = Date.now();
await pSeries([
  async () => { await new Promise(r => setTimeout(r, 10)); },
  async () => { await new Promise(r => setTimeout(r, 10)); },
]);
lines.push("3 sequential_ok=" + (Date.now() - t1 >= 18));

let err = null;
try {
  await pSeries([async () => 1, async () => { throw new Error("boom"); }, async () => 3]);
} catch (e) { err = e.message; }
lines.push("4 err=" + err);

const r5 = await pSeries([]);
lines.push("5 empty=" + JSON.stringify(r5));

lines.push("6 isFn=" + (typeof pSeries === "function"));

process.stdout.write(lines.join("\n") + "\n");
