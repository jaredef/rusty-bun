import { parse } from "ndjson";
import { Readable } from "node:stream";
const out = [];
const src = Readable.from(['{"a":1}\n{"b":2}\n']);
src.pipe(parse()).on("data", obj => out.push(obj));
await new Promise(r => setTimeout(r, 50));
process.stdout.write(JSON.stringify({ count: out.length, first: out[0] }) + "\n");
