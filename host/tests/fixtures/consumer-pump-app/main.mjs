import pump from "pump";
import { Readable, Writable } from "node:stream";
const got = [];
const src = Readable.from(["a", "b", "c"]);
const sink = new Writable({ objectMode: true, write(chunk, enc, cb) { got.push(chunk.toString()); cb(); } });
await new Promise(r => pump(src, sink, () => r()));
process.stdout.write(JSON.stringify({ got }) + "\n");
