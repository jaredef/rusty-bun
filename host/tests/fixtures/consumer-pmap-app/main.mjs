// p-map ^7 — async map with concurrency limit (Sindresorhus, distinct
// from p-limit).
import pMap from "p-map";

const lines = [];

async function main() {
  const r1 = await pMap([1, 2, 3, 4, 5], async n => n * 10, { concurrency: 2 });
  lines.push("1 " + JSON.stringify(r1));

  const order = [];
  await pMap([10, 5, 20, 15], async n => {
    await new Promise(r => setTimeout(r, n));
    order.push(n);
  }, { concurrency: 1 });
  lines.push("2 " + JSON.stringify(order));

  const r3 = await pMap([1, 2, 3], async n => n + 100, { concurrency: 100 });
  lines.push("3 " + JSON.stringify(r3));

  let err = null;
  try {
    await pMap([1, 2, 3], async n => { if (n === 2) throw new Error("boom"); return n; });
  } catch (e) { err = e.message; }
  lines.push("4 err=" + err);

  const r5 = await pMap([], async n => n, { concurrency: 2 });
  lines.push("5 empty=" + JSON.stringify(r5));

  // stopOnError false
  const r6 = await pMap([1, 2, 3, 4], async n => {
    if (n === 2 || n === 3) throw new Error("e" + n);
    return n;
  }, { concurrency: 2, stopOnError: false }).catch(e => e.errors.length);
  lines.push("6 errsCollected=" + r6);
}

await main();
process.stdout.write(lines.join("\n") + "\n");
