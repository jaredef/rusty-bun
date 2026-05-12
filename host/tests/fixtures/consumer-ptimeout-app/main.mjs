// p-timeout ^7 — reject a promise if it takes too long.
import pTimeout, { TimeoutError } from "p-timeout";

const lines = [];

async function main() {
  const r1 = await pTimeout(Promise.resolve(42), { milliseconds: 1000 });
  lines.push("1 fast=" + r1);

  let err = null;
  try {
    await pTimeout(new Promise(r => setTimeout(() => r("late"), 200)), { milliseconds: 30 });
  } catch (e) { err = e instanceof TimeoutError ? "timeout" : e.message; }
  lines.push("2 timed=" + err);

  let err2 = null;
  try {
    await pTimeout(new Promise(r => setTimeout(r, 200)), { milliseconds: 30, message: "custom-msg" });
  } catch (e) { err2 = e.message; }
  lines.push("3 custom=" + err2);

  let fbVal = null;
  fbVal = await pTimeout(new Promise(r => setTimeout(() => r("never"), 200)), { milliseconds: 30, fallback: () => "fb" });
  lines.push("4 fallback=" + fbVal);

  lines.push("5 isFn=" + (typeof pTimeout === "function"));

  // Infinite milliseconds → never times out
  const r6 = await pTimeout(Promise.resolve("inf"), { milliseconds: Infinity });
  lines.push("6 inf=" + r6);
}

await main();
process.stdout.write(lines.join("\n") + "\n");
