// p-some ^7 — resolve when N of M promises resolve.
import pSome from "p-some";

const lines = [];

async function main() {
  const r1 = await pSome([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)], { count: 2 });
  lines.push("1 " + JSON.stringify(r1.sort((a, b) => a - b)));

  let err = null;
  try {
    await pSome([Promise.reject("a"), Promise.reject("b")], { count: 1 });
  } catch (e) { err = e.constructor.name; }
  lines.push("2 err=" + err);

  const r3 = await pSome([Promise.reject("a"), Promise.resolve("b"), Promise.resolve("c")], { count: 1 });
  lines.push("3 " + JSON.stringify(r3));

  const r4 = await pSome([Promise.resolve(1)], { count: 1 });
  lines.push("4 " + JSON.stringify(r4));

  lines.push("5 isFn=" + (typeof pSome === "function"));

  // filter
  const r6 = await pSome([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)], {
    count: 2,
    filter: x => x > 1,
  });
  lines.push("6 " + JSON.stringify(r6.sort((a, b) => a - b)));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
