// eventemitter2 ^6 — namespaced/wildcard event emitter (distinct from
// node:events and eventemitter3).
import { EventEmitter2 } from "eventemitter2";

const lines = [];
const e = new EventEmitter2({ wildcard: true, delimiter: "." });

const out = [];
e.on("foo.*", function (v) { out.push("f:" + this.event + ":" + v); });
e.emit("foo.bar", 1);
e.emit("foo.baz", 2);
lines.push("1 " + JSON.stringify(out));

const ns = [];
e.on("a.b.c", v => ns.push("abc:" + v));
e.on("a.b.*", v => ns.push("abs:" + v));
e.emit("a.b.c", 42);
lines.push("2 " + JSON.stringify(ns));

const oc = [];
e.once("o", v => oc.push(v));
e.emit("o", "x"); e.emit("o", "y");
lines.push("3 " + JSON.stringify(oc));

const many = [];
e.many("m", 2, v => many.push(v));
e.emit("m", 1); e.emit("m", 2); e.emit("m", 3);
lines.push("4 " + JSON.stringify(many));

const lst = e.listeners("foo.bar");
lines.push("5 listeners=" + lst.length);

const counts = e.listenerCount("a.b.c");
lines.push("6 abcCount=" + counts);

process.stdout.write(lines.join("\n") + "\n");
