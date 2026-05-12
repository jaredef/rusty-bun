// circular-json ^0.5 — JSON with cycle preservation (round-trip).
import CircularJSON from "circular-json";

const lines = [];

const a = { x: 1 };
a.self = a;
const s = CircularJSON.stringify(a);
const p = CircularJSON.parse(s);
lines.push("1 cycRT=" + (p.self === p));

const b = { items: [], name: "list" };
b.items.push(b);
const s2 = CircularJSON.stringify(b);
const p2 = CircularJSON.parse(s2);
lines.push("2 arrCyc=" + (p2.items[0] === p2));

lines.push("3 " + CircularJSON.stringify({ x: 1, y: 2 }));
lines.push("4 " + JSON.stringify(CircularJSON.parse('{"a":1,"b":2}')));

const c = { a: { b: { c: 1 } } };
const c2 = CircularJSON.parse(CircularJSON.stringify(c));
lines.push("5 deep=" + JSON.stringify(c2));

lines.push("6 strRT=" + (CircularJSON.parse(CircularJSON.stringify("plain")) === "plain"));

process.stdout.write(lines.join("\n") + "\n");
