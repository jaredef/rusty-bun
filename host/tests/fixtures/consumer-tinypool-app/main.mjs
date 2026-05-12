// tiny-typed-emitter ^2 — TS-typed EventEmitter wrapper.
import { TypedEmitter } from "tiny-typed-emitter";

const lines = [];
const e = new TypedEmitter();

const out = [];
e.on("hello", (a, b) => out.push("h:" + a + ":" + b));
e.emit("hello", "world", 42);
e.emit("hello", "again", 99);
lines.push("1 " + JSON.stringify(out));

const once = [];
e.once("x", v => once.push(v));
e.emit("x", 1); e.emit("x", 2);
lines.push("2 " + JSON.stringify(once));

const cnt = [];
const fn = v => cnt.push(v);
e.on("y", fn);
e.emit("y", "a");
e.off("y", fn);
e.emit("y", "b");
lines.push("3 " + JSON.stringify(cnt));

lines.push("4 listeners=" + e.listenerCount("hello"));
e.removeAllListeners("hello");
lines.push("5 cleared=" + e.listenerCount("hello"));

lines.push("6 isEE=" + (typeof e.on === "function"));

process.stdout.write(lines.join("\n") + "\n");
