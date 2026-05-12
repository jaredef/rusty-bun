// tiny-emitter ^2 — small event emitter (CJS-default-as-import).
import Emitter from "tiny-emitter";

const lines = [];
const e = new Emitter();

const out = [];
e.on("evt", (a, b) => out.push("evt:" + a + ":" + b));
e.emit("evt", 1, 2);
e.emit("evt", 3, 4);
lines.push("1 " + JSON.stringify(out));

const once = [];
e.once("o", v => once.push(v));
e.emit("o", "a"); e.emit("o", "b");
lines.push("2 " + JSON.stringify(once));

const cnt = [];
const fn = v => cnt.push(v);
e.on("c", fn);
e.emit("c", "x");
e.off("c", fn);
e.emit("c", "y");
lines.push("3 " + JSON.stringify(cnt));

const all = [];
e.on("m", v => all.push("a:" + v));
e.on("m", v => all.push("b:" + v));
e.emit("m", 1);
lines.push("4 " + JSON.stringify(all));

lines.push("5 hasOn=" + (typeof e.on === "function") + " hasEmit=" + (typeof e.emit === "function"));

const ctx = { tag: "self" };
e.on("ctx", function () { lines.push("6 ctx=" + this.tag); }, ctx);
e.emit("ctx");

process.stdout.write(lines.join("\n") + "\n");
