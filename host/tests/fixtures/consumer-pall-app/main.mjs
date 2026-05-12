// p-all ^5 — run promise-returning fns concurrently (similar to p-map
// but takes fn array directly).
import pAll from "p-all";

const lines = [];

async function main() {
  const r1 = await pAll([
    () => Promise.resolve(1),
    () => Promise.resolve(2),
    () => Promise.resolve(3),
  ]);
  lines.push("1 " + JSON.stringify(r1));

  const r2 = await pAll([
    () => new Promise(r => setTimeout(() => r("a"), 10)),
    () => new Promise(r => setTimeout(() => r("b"), 5)),
  ], { concurrency: 1 });
  lines.push("2 " + JSON.stringify(r2));

  const r3 = await pAll([() => 42], { concurrency: 10 });
  lines.push("3 " + JSON.stringify(r3));

  let err = null;
  try {
    await pAll([
      () => Promise.resolve(1),
      () => Promise.reject(new Error("boom")),
      () => Promise.resolve(3),
    ]);
  } catch (e) { err = e.message; }
  lines.push("4 err=" + err);

  // stopOnError false
  const settled = await pAll([
    () => Promise.resolve(1),
    () => Promise.reject(new Error("e2")),
    () => Promise.resolve(3),
  ], { stopOnError: false }).catch(e => e.errors.map(x => x.message));
  lines.push("5 errs=" + JSON.stringify(settled));

  const r6 = await pAll([], { concurrency: 5 });
  lines.push("6 empty=" + JSON.stringify(r6));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
