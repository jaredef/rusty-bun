import pako from "pako";
const raw = new TextEncoder().encode("hello hello hello hello");
const c = pako.deflate(raw);
const d = new TextDecoder().decode(pako.inflate(c));
process.stdout.write(JSON.stringify({ cLen: c.length, d }) + "\n");
