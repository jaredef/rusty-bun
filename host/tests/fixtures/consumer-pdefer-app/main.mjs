// p-defer ^4 — externally-controllable Promise.
import pDefer from "p-defer";

const lines = [];

async function main() {
  const d1 = pDefer();
  setTimeout(() => d1.resolve("ok"), 5);
  lines.push("1 " + (await d1.promise));

  const d2 = pDefer();
  setTimeout(() => d2.reject(new Error("nope")), 5);
  let err = null;
  try { await d2.promise; } catch (e) { err = e.message; }
  lines.push("2 err=" + err);

  const d3 = pDefer();
  d3.resolve(42);
  lines.push("3 sync=" + await d3.promise);

  const d4 = pDefer();
  lines.push("4 hasFns=" + (typeof d4.resolve === "function") + "/" + (typeof d4.reject === "function"));
  d4.resolve();

  const d5 = pDefer();
  setTimeout(() => d5.resolve([1, 2, 3]), 5);
  lines.push("5 arr=" + JSON.stringify(await d5.promise));

  const d6 = pDefer();
  d6.resolve({ a: 1 });
  lines.push("6 obj=" + JSON.stringify(await d6.promise));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
