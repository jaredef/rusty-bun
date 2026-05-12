// async ^3 — async control-flow library (callback-style, distinct
// from promise-based p-limit/async-pool).
import async from "async";

const lines = [];

async function main() {
  const r1 = await new Promise(res => async.map([1, 2, 3], (n, cb) => cb(null, n * 2), (e, results) => res(results)));
  lines.push("1 " + JSON.stringify(r1));

  const r2 = await new Promise(res => async.filter([1, 2, 3, 4, 5], (n, cb) => cb(null, n % 2 === 0), (e, results) => res(results)));
  lines.push("2 " + JSON.stringify(r2));

  const r3 = await new Promise(res => async.reduce([1, 2, 3, 4], 0, (acc, n, cb) => cb(null, acc + n), (e, result) => res(result)));
  lines.push("3 " + r3);

  const order = [];
  await new Promise(res => async.series([
    cb => { order.push("a"); cb(null); },
    cb => { order.push("b"); cb(null); },
    cb => { order.push("c"); cb(null); },
  ], () => res()));
  lines.push("4 " + order.join(","));

  const r5 = await new Promise(res => async.parallel({
    one: cb => cb(null, 1),
    two: cb => cb(null, 2),
  }, (e, results) => res(results)));
  lines.push("5 " + JSON.stringify(r5));

  const r6 = await new Promise(res => async.times(3, (n, cb) => cb(null, n * 10), (e, results) => res(results)));
  lines.push("6 " + JSON.stringify(r6));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
