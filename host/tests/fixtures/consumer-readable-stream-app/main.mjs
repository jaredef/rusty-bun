import { Transform } from "readable-stream";
const t = new Transform({
  objectMode: true,
  transform(c, _e, cb) { cb(null, c * 2); },
});
const out = [];
t.on("data", x => out.push(x));
t.write(1); t.write(2); t.write(3); t.end();
await new Promise(r => setTimeout(r, 20));
process.stdout.write(JSON.stringify({ out }) + "\n");
