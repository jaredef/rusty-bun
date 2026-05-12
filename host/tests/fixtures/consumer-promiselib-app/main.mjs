// bluebird ^3 — classic Promise library (Promise.map/each/props/etc).
import Promise from "bluebird";

const lines = [];

async function main() {
  const r1 = await Promise.map([1, 2, 3], async n => n * 10);
  lines.push("1 " + JSON.stringify(r1));

  const r2 = await Promise.props({ a: Promise.resolve(1), b: Promise.resolve(2) });
  lines.push("2 " + JSON.stringify(r2));

  const order = [];
  await Promise.each([10, 5, 20], async n => { order.push(n); });
  lines.push("3 " + JSON.stringify(order));

  const r4 = await Promise.reduce([1, 2, 3, 4], async (acc, n) => acc + n, 0);
  lines.push("4 sum=" + r4);

  const r5 = await Promise.any([Promise.reject("a"), Promise.resolve("b"), Promise.resolve("c")]);
  lines.push("5 any=" + r5);

  const r6 = await Promise.filter([1, 2, 3, 4, 5], async n => n % 2 === 0);
  lines.push("6 " + JSON.stringify(r6));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
