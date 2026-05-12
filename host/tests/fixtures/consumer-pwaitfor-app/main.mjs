import pWaitFor from "p-wait-for";

let counter = 0;
const tick = () => { counter++; return counter >= 3; };

await pWaitFor(tick, { interval: 5 });
const reachedTarget = counter;

let timedOut = false;
try {
  await pWaitFor(() => false, { interval: 5, timeout: 50 });
} catch (e) {
  timedOut = e && (e.name === "TimeoutError" || /timed?\s*out/i.test(e.message || ""));
}

process.stdout.write(JSON.stringify({
  reachedTarget,
  timedOut,
}) + "\n");
