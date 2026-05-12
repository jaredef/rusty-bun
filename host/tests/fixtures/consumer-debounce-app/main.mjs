// throttle-debounce ^5 — rate-limit fn invocation.
import { throttle, debounce } from "throttle-debounce";

const lines = [];

async function main() {
  // 1: throttle — first call passes through immediately
  let n = 0;
  const inc = throttle(50, () => { n++; });
  inc(); inc(); inc(); inc();
  await new Promise(r => setTimeout(r, 100));
  lines.push("1 throttleCount=" + n);

  // 2: debounce — only trailing call fires
  let m = 0;
  let lastArg = null;
  const d = debounce(30, (x) => { m++; lastArg = x; });
  d("a"); d("b"); d("c");
  await new Promise(r => setTimeout(r, 80));
  lines.push("2 debounceCount=" + m + " last=" + lastArg);

  // 3: cancel debounce
  let k = 0;
  const d2 = debounce(40, () => { k++; });
  d2(); d2.cancel();
  await new Promise(r => setTimeout(r, 80));
  lines.push("3 cancelled=" + k);

  // 4: debounce atBegin (leading)
  let p = 0;
  const d3 = debounce(30, () => { p++; }, { atBegin: true });
  d3(); d3(); d3();
  await new Promise(r => setTimeout(r, 60));
  lines.push("4 atBegin=" + p);

  // 5: throttle noLeading
  let q = 0;
  const t = throttle(30, () => { q++; }, { noLeading: true });
  t(); t(); t();
  await new Promise(r => setTimeout(r, 80));
  lines.push("5 noLeading=" + q);

  // 6: shape check
  lines.push("6 isFn=" + (typeof debounce(10, () => {}) === "function"));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
