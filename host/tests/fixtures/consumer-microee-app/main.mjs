// microee ^0.0 — tiny event emitter mixin pattern (distinct from
// tiny-emitter/eventemitter2/eventemitter3).
import MicroEE from "microee";

const lines = [];

function Foo() {}
MicroEE.mixin(Foo);
const f = new Foo();

const out = [];
f.on("x", v => out.push(v));
f.emit("x", 1);
f.emit("x", 2);
lines.push("1 " + JSON.stringify(out));

const once = [];
f.once("o", v => once.push(v));
f.emit("o", "a"); f.emit("o", "b");
lines.push("2 " + JSON.stringify(once));

const cnt = [];
const fn = v => cnt.push(v);
f.on("c", fn);
f.emit("c", 1);
f.removeListener("c", fn);
f.emit("c", 2);
lines.push("3 " + JSON.stringify(cnt));

const all = [];
f.on("m", v => all.push("a:" + v));
f.on("m", v => all.push("b:" + v));
f.emit("m", 1);
lines.push("4 " + JSON.stringify(all));

lines.push("5 hasOn=" + (typeof f.on === "function") + " hasEmit=" + (typeof f.emit === "function"));

const g = new Foo();
g.on("e", v => lines.push("6 separate=" + v));
g.emit("e", "ok");

process.stdout.write(lines.join("\n") + "\n");
