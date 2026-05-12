// eventemitter3 ^5 — third-party EventEmitter, distinct from node:events.
// Used by socket.io + many browser-compat libs. Pure CJS through bridge.
import EventEmitter from "eventemitter3";

const lines = [];

// 1: basic on + emit
{
  const ee = new EventEmitter();
  let got = null;
  ee.on("foo", (a, b) => { got = a + ":" + b; });
  ee.emit("foo", "hello", 42);
  lines.push("1 got=" + got);
}

// 2: once fires once
{
  const ee = new EventEmitter();
  let n = 0;
  ee.once("x", () => n++);
  ee.emit("x"); ee.emit("x"); ee.emit("x");
  lines.push("2 n=" + n);
}

// 3: off removes
{
  const ee = new EventEmitter();
  let n = 0;
  const h = () => n++;
  ee.on("y", h);
  ee.emit("y"); ee.off("y", h); ee.emit("y");
  lines.push("3 n=" + n);
}

// 4: multiple listeners + listenerCount
{
  const ee = new EventEmitter();
  let s = "";
  ee.on("z", () => s += "A");
  ee.on("z", () => s += "B");
  ee.on("z", () => s += "C");
  ee.emit("z");
  lines.push("4 s=" + s + " count=" + ee.listenerCount("z"));
}

// 5: removeAllListeners
{
  const ee = new EventEmitter();
  ee.on("p", () => {});
  ee.on("q", () => {});
  ee.removeAllListeners();
  lines.push("5 pCount=" + ee.listenerCount("p") + " qCount=" + ee.listenerCount("q"));
}

// 6: eventNames
{
  const ee = new EventEmitter();
  ee.on("a", () => {});
  ee.on("b", () => {});
  ee.on("c", () => {});
  lines.push("6 names=" + JSON.stringify(ee.eventNames().sort()));
}

// 7: error-event NO throw without handler (distinct from node:events)
{
  const ee = new EventEmitter();
  let didEmit = false;
  try { didEmit = ee.emit("error", new Error("nobody")); }
  catch (_) { didEmit = "threw"; }
  lines.push("7 didEmit=" + didEmit);
}

process.stdout.write(lines.join("\n") + "\n");
