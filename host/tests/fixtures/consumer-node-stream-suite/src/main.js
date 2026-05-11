// Tier-J consumer #59: node:stream. Tier-Π3.9 closure round.
//
// Strategy: exercise the patterns real npm packages use — Readable.from
// for adapting iterables, on('data')+on('end') for the canonical
// consume-the-stream loop, PassThrough as a relay, Transform for
// uppercase-style filters, pipeline for error-propagating chains,
// async iteration via Symbol.asyncIterator.

import { Readable, Writable, Transform, PassThrough, pipeline } from "node:stream";

function collect(stream) {
    return new Promise((resolve, reject) => {
        const chunks = [];
        stream.on("data", (c) => chunks.push(c));
        stream.on("end", () => resolve(chunks));
        stream.on("error", reject);
    });
}

async function selfTest() {
    const results = [];

    // 1. Readable.from(array) emits each item then ends.
    const r1 = Readable.from(["a", "b", "c"]);
    const chunks1 = (await collect(r1)).map(String);
    results.push(["readable-from-array",
        chunks1.length === 3 && chunks1[0] === "a" && chunks1[2] === "c"]);

    // 2. Readable.from(asyncIterable).
    async function* gen() { yield 1; yield 2; yield 3; }
    const r2 = Readable.from(gen());
    const chunks2 = await collect(r2);
    results.push(["readable-from-async-iterable",
        chunks2.length === 3 && Number(chunks2[0]) === 1 && Number(chunks2[2]) === 3]);

    // 3. for await...of consumes a Readable.
    const r3 = Readable.from(["x", "y", "z"]);
    const out3 = [];
    for await (const chunk of r3) out3.push(String(chunk));
    results.push(["readable-async-iteration",
        out3.length === 3 && out3[0] === "x" && out3[2] === "z"]);

    // 4. Writable collects chunks; finish fires after end(). Strings
    //    are encoded to Buffer in canonical node-stream mode (Bun matches).
    const collected4 = [];
    const w4 = new Writable({
        write(chunk, enc, cb) { collected4.push(String(chunk)); cb(); }
    });
    await new Promise((resolve) => {
        w4.on("finish", resolve);
        w4.write("alpha");
        w4.write("beta");
        w4.end();
    });
    results.push(["writable-collects",
        collected4.length === 2 && collected4[0] === "alpha"]);

    // 5. Transform uppercase filter via pipe. String() coerce in case
    //    chunks arrive as Buffer in canonical node-stream mode.
    const upper = new Transform({
        transform(chunk, enc, cb) { cb(null, String(chunk).toUpperCase()); }
    });
    Readable.from(["hello", "world"]).pipe(upper);
    const chunks5 = (await collect(upper)).map(String);
    results.push(["transform-uppercase",
        chunks5.length === 2 && chunks5[0] === "HELLO" && chunks5[1] === "WORLD"]);

    // 6. PassThrough relays unchanged (modulo Buffer coercion).
    const pt = new PassThrough();
    Readable.from(["p", "q"]).pipe(pt);
    const chunks6 = (await collect(pt)).map(String);
    results.push(["passthrough-relays",
        chunks6.length === 2 && chunks6[0] === "p" && chunks6[1] === "q"]);

    // 7. pipeline with error propagation.
    const pipelineResult = await new Promise((resolve) => {
        const collected = [];
        const w = new Writable({ write(c, e, cb) { collected.push(String(c)); cb(); } });
        pipeline(
            Readable.from(["one", "two"]),
            new Transform({ transform(c, e, cb) { cb(null, "[" + String(c) + "]"); } }),
            w,
            (err) => resolve({ err, collected })
        );
    });
    results.push(["pipeline-chain",
        !pipelineResult.err &&
        pipelineResult.collected.length === 2 &&
        pipelineResult.collected[0] === "[one]"]);

    // 8. Subclassing Readable via the options-callback constructor pattern.
    let pushed = 0;
    const r8 = new Readable({
        read(_size) {
            pushed += 1;
            if (pushed <= 3) this.push("item-" + pushed);
            else this.push(null);
        }
    });
    const chunks8 = (await collect(r8)).map(String);
    results.push(["readable-subclass-via-options",
        chunks8.length === 3 && chunks8[0] === "item-1" && chunks8[2] === "item-3"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
