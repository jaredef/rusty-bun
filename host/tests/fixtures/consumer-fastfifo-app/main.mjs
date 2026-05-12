// fast-fifo ^1 — chunked-array FIFO queue (used by streamx, mafintosh
// libs). Distinct axis: high-throughput queue with chunk recycling.
import FastFIFO from "fast-fifo";

const lines = [];

const q = new FastFIFO();
for (let i = 0; i < 5; i++) q.push(i);

lines.push("1 isEmpty=" + q.isEmpty());

const out = [];
while (!q.isEmpty()) out.push(q.shift());
lines.push("2 drained=" + JSON.stringify(out));
lines.push("3 emptyNow=" + q.isEmpty());

// Push then shift interleaved
q.push("a"); q.push("b");
const a = q.shift();
q.push("c");
const b = q.shift();
const c = q.shift();
lines.push("4 interleave=" + [a, b, c].join(","));

// Large
for (let i = 0; i < 1000; i++) q.push(i);
let count = 0;
while (!q.isEmpty()) { q.shift(); count++; }
lines.push("5 large=" + count);

// peek
q.push(42);
lines.push("6 peek=" + q.peek() + " stillThere=" + (q.peek() === 42));

process.stdout.write(lines.join("\n") + "\n");
