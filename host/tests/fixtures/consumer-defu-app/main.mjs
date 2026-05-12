// defu ^6 — recursive defaults merger (unjs).
import { defu } from "defu";

const lines = [];
lines.push("1 " + JSON.stringify(defu({ a: 1 }, { b: 2 })));
lines.push("2 " + JSON.stringify(defu({ a: 1 }, { a: 99, b: 2 })));
lines.push("3 " + JSON.stringify(defu({ x: { y: 1 } }, { x: { z: 2 } })));
lines.push("4 " + JSON.stringify(defu({ arr: [1, 2] }, { arr: [3, 4] })));
lines.push("5 " + JSON.stringify(defu({}, { a: { b: { c: 1 } } })));
lines.push("6 " + JSON.stringify(defu({ a: null }, { a: { b: 1 } })));

process.stdout.write(lines.join("\n") + "\n");
