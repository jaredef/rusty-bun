// streamx ^2 — Mafintosh's modern stream lib.
import { Readable, Writable, Transform } from "streamx";

const lines = [];

async function main() {
  const r1 = await new Promise(resolve => {
    const out = [];
    const r = Readable.from([1, 2, 3]);
    r.on("data", v => out.push(v));
    r.on("end", () => resolve(out));
  });
  lines.push("1 " + JSON.stringify(r1));

  const r2 = await new Promise(resolve => {
    const got = [];
    const w = new Writable({
      write(d, cb) { got.push(d); cb(null); }
    });
    Readable.from([10, 20, 30]).pipe(w);
    w.on("finish", () => resolve(got));
  });
  lines.push("2 " + JSON.stringify(r2));

  const r3 = await new Promise(resolve => {
    const out = [];
    const t = new Transform({
      transform(d, cb) { cb(null, d * 2); }
    });
    Readable.from([1, 2, 3]).pipe(t);
    t.on("data", v => out.push(v));
    t.on("end", () => resolve(out));
  });
  lines.push("3 " + JSON.stringify(r3));

  lines.push("4 isFn=" + (typeof Readable === "function"));
  lines.push("5 hasWrite=" + (typeof Writable === "function"));
  lines.push("6 hasTransform=" + (typeof Transform === "function"));
}

await main();
process.stdout.write(lines.join("\n") + "\n");
