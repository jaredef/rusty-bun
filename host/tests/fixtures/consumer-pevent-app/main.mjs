// p-event ^7 — promisify event emitter events.
import { pEvent, pEventMultiple, pEventIterator } from "p-event";
import { EventEmitter } from "node:events";

const lines = [];

async function main() {
  const e = new EventEmitter();
  setTimeout(() => e.emit("done", 42), 5);
  const r1 = await pEvent(e, "done");
  lines.push("1 r=" + r1);

  const e2 = new EventEmitter();
  setTimeout(() => { e2.emit("x", 1); e2.emit("x", 2); e2.emit("x", 3); }, 5);
  const r2 = await pEventMultiple(e2, "x", { count: 3 });
  lines.push("2 " + JSON.stringify(r2));

  const e3 = new EventEmitter();
  setTimeout(() => e3.emit("err", new Error("boom")), 5);
  let err = null;
  try {
    await pEvent(e3, "done", { rejectionEvents: ["err"] });
  } catch (e) { err = e.message; }
  lines.push("3 err=" + err);

  const e4 = new EventEmitter();
  const promise = pEvent(e4, "done", { timeout: 30 });
  let err4 = null;
  try { await promise; } catch (e) { err4 = e.name; }
  lines.push("4 timeoutErr=" + err4);

  const e5 = new EventEmitter();
  setTimeout(() => { e5.emit("evt", 10); e5.emit("evt", 20); }, 5);
  const r5 = await pEvent(e5, "evt", { filter: v => v > 15 });
  lines.push("5 filtered=" + r5);

  lines.push("6 isFn=" + (typeof pEvent === "function"));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
