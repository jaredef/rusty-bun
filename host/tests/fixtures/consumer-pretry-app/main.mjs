// p-retry ^8 — retry a promise-returning fn on failure.
import pRetry from "p-retry";

const lines = [];

async function main() {
  let calls1 = 0;
  const r1 = await pRetry(async () => {
    calls1++;
    if (calls1 < 3) throw new Error("retry me");
    return "ok";
  }, { retries: 5, minTimeout: 1 });
  lines.push("1 calls=" + calls1 + " val=" + r1);

  let err = null;
  let calls2 = 0;
  try {
    await pRetry(async () => { calls2++; throw new Error("always fails"); }, { retries: 2, minTimeout: 1 });
  } catch (e) { err = e.message; }
  lines.push("2 exhausted calls=" + calls2 + " err=" + err);

  let calls3 = 0;
  const r3 = await pRetry(async () => "first-try", { retries: 5, minTimeout: 1 });
  calls3 = 1;
  lines.push("3 firstShot=" + r3);

  let onFailedCount = 0;
  let calls4 = 0;
  try {
    await pRetry(async () => { calls4++; throw new Error("fail" + calls4); }, {
      retries: 2,
      minTimeout: 1,
      onFailedAttempt: () => { onFailedCount++; }
    });
  } catch {}
  lines.push("4 onFailed=" + onFailedCount);

  lines.push("5 isFn=" + (typeof pRetry === "function"));

  let abortedAt = 0;
  let err6 = null;
  try {
    await pRetry(async (attemptNum) => {
      abortedAt = attemptNum;
      const { AbortError } = await import("p-retry");
      throw new AbortError("stop now");
    }, { retries: 5, minTimeout: 1 });
  } catch (e) { err6 = e.message; }
  lines.push("6 aborted at=" + abortedAt + " err=" + err6);
}

await main();
process.stdout.write(lines.join("\n") + "\n");
