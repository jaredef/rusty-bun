// map-obj ^6 — transform object keys/values.
import mapObj from "map-obj";

const lines = [];
lines.push("1 " + JSON.stringify(mapObj({ a: 1, b: 2 }, (k, v) => [k.toUpperCase(), v * 10])));
lines.push("2 " + JSON.stringify(mapObj({ a: 1 }, (k, v) => [k, v + 100])));
lines.push("3 " + JSON.stringify(mapObj({ a: { b: 1 } }, (k, v) => [k, v], { deep: true })));
lines.push("4 " + JSON.stringify(mapObj({ a: { b: 1, c: 2 } }, (k, v) => [k.toUpperCase(), v], { deep: true })));
lines.push("5 " + JSON.stringify(mapObj({ x: 1, y: 2, z: 3 }, (k, v) => [k, v < 3 ? v : undefined])));
lines.push("6 " + JSON.stringify(mapObj({}, (k, v) => [k, v])));

process.stdout.write(lines.join("\n") + "\n");
