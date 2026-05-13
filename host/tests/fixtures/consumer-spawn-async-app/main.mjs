// Π2.6.d.b: Bun.spawn with real async pipes + mio reactor.
//
// Spawns `cat`, writes to its stdin in two chunks, closes stdin,
// drains stdout via the ReadableStream reader API, awaits exit.
// Exercises the concurrent-write-stdin / read-stdout / wait-exit
// pattern that the previous spawnSync-wrapping facade couldn't
// support.

const proc = Bun.spawn(["cat"], { stdin: "pipe", stdout: "pipe" });

proc.stdin.write("hello from rusty-bun\n");
proc.stdin.write("second line\n");
proc.stdin.end();

const reader = proc.stdout.getReader();
const parts = [];
let total = 0;
for (;;) {
  const { value, done } = await reader.read();
  if (done) break;
  parts.push(value);
  total += value.length;
}
const merged = new Uint8Array(total);
let off = 0;
for (const p of parts) { merged.set(p, off); off += p.length; }
const out = new TextDecoder().decode(merged);
const exitCode = await proc.exited;

process.stdout.write(JSON.stringify({
  exitCode,
  outLen: out.length,
  hasHello: out.includes("hello from rusty-bun"),
  hasSecond: out.includes("second line"),
}) + "\n");
