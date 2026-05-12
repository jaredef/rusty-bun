// yallist ^5 — doubly linked list (used by isaacs/lru-cache).
import { Yallist } from "yallist";

const lines = [];
const l = new Yallist();
l.push(1); l.push(2); l.push(3);
lines.push("1 len=" + l.length + " head=" + l.head.value + " tail=" + l.tail.value);

const popped = l.pop();
lines.push("2 popped=" + popped + " len=" + l.length);

l.unshift(0);
lines.push("3 head=" + l.head.value + " toArr=" + JSON.stringify(l.toArray()));

const l2 = Yallist.create([10, 20, 30]);
const doubled = l2.map(x => x * 2);
lines.push("4 " + JSON.stringify(doubled.toArray()));

const sum = Yallist.create([1, 2, 3, 4]).reduce((a, b) => a + b);
lines.push("5 sum=" + sum);

const l3 = Yallist.create([1, 2, 3, 4, 5]);
const collected = [];
l3.forEach(v => { if (v > 2) collected.push(v); });
lines.push("6 " + JSON.stringify(collected));

process.stdout.write(lines.join("\n") + "\n");
