import split2 from "split2";
import { Readable } from "node:stream";
const out = [];
const src = Readable.from(["line1\nline2\nline3\n"]);
src.pipe(split2()).on("data", l => out.push(l.toString()));
await new Promise(r => setTimeout(r, 50));
process.stdout.write(JSON.stringify({ lines: out }) + "\n");
