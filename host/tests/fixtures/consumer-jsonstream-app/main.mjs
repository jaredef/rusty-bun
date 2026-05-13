import JSONStream from "JSONStream";
import { Readable } from "node:stream";
const out = [];
const src = Readable.from(['{"items":[{"a":1},{"b":2}]}']);
src.pipe(JSONStream.parse("items.*")).on("data", x => out.push(x));
await new Promise(r => setTimeout(r, 100));
process.stdout.write(JSON.stringify({ items: out }) + "\n");
