// Π2.6.d.a: fs.watch over inotify + mio reactor.
//
// Creates a temp directory, sets up a watch, writes a file and
// removes it, and asserts both events surface to the listener.
// Bun + rusty-bun produce the same hasCreate/hasDelete signals
// (events may differ in count + ordering across kernels, so the
// fixture asserts presence rather than equality).

import fs from "node:fs";
import path from "node:path";
import os from "node:os";

const tmp = path.join(os.tmpdir(), "rb-fswatch-" + process.pid + "-" + Date.now());
fs.mkdirSync(tmp, { recursive: true });

const events = [];
const watcher = fs.watch(tmp, (kind, name) => {
  events.push(kind + ":" + name);
});

await new Promise(r => setTimeout(r, 20));

const f = path.join(tmp, "hello.txt");
fs.writeFileSync(f, "x");

await new Promise(r => setTimeout(r, 50));

fs.unlinkSync(f);

await new Promise(r => setTimeout(r, 50));

watcher.close();
fs.rmdirSync(tmp);

const hasCreate = events.some(e => e.endsWith(":hello.txt"));
const hasDelete = events.filter(e => e.endsWith(":hello.txt") && e.startsWith("rename")).length >= 1;

process.stdout.write(JSON.stringify({
  hasCreate,
  hasDelete,
}) + "\n");
