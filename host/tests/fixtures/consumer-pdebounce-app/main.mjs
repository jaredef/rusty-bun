// p-debounce ^5 — promise-aware debounce.
import pDebounce from "p-debounce";

const lines = [];

async function main() {
  let calls = 0;
  const fn = pDebounce(async (x) => { calls++; return x * 2; }, 20);
  const [a, b, c] = await Promise.all([fn(1), fn(2), fn(3)]);
  lines.push("1 calls=" + calls + " a=" + a + " b=" + b + " c=" + c);

  let calls2 = 0;
  const fn2 = pDebounce(async (x) => { calls2++; return x; }, 20);
  await fn2(10);
  await new Promise(r => setTimeout(r, 30));
  await fn2(20);
  lines.push("2 sequential calls=" + calls2);

  let calls3 = 0;
  const fn3 = pDebounce.promise(async (x) => { calls3++; return x; });
  await Promise.all([fn3(1), fn3(2), fn3(3)]);
  lines.push("3 promiseVariant calls=" + calls3);

  let calls4 = 0;
  const fn4 = pDebounce(async (x) => { calls4++; return x; }, 20, { leading: true });
  await fn4(99);
  await new Promise(r => setTimeout(r, 5));
  await fn4(100);
  await new Promise(r => setTimeout(r, 30));
  lines.push("4 leading calls=" + calls4);

  lines.push("5 isFn=" + (typeof pDebounce === "function"));

  let calls6 = 0;
  const fn6 = pDebounce(async (x) => { calls6++; return x; }, 10);
  for (let i = 0; i < 5; i++) fn6(i);
  await new Promise(r => setTimeout(r, 30));
  lines.push("6 burstCalls=" + calls6);
}

await main();
process.stdout.write(lines.join("\n") + "\n");
