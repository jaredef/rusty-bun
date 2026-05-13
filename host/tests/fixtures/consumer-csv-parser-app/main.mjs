import csv from "csv-parser";
import { Readable } from "node:stream";
const out = [];
const src = Readable.from(["name,age\nAlice,30\nBob,25\n"]);
src.pipe(csv()).on("data", r => out.push(r));
await new Promise(r => setTimeout(r, 50));
process.stdout.write(JSON.stringify({ rows: out }) + "\n");
