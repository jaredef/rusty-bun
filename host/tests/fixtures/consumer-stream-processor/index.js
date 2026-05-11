// Tier-J consumer #2 (CJS, post-M8 reconciliation): Bun-portable
// stream-processing pipeline. Exercises ReadableStream + TransformStream
// + WritableStream + AbortController + setTimeout + node:fs + node:Buffer
// + URL across CJS module boundaries.
//
// Bun-portability achieved via three reconciliations (M8(a) alignments):
//   - require("node:fs") instead of global fs (rusty-bun-host's CJS
//     loader maps node:fs to the wired fs global)
//   - Buffer.from(...).toString("hex") instead of Buffer.encodeHex (the
//     Buffer class wraps Uint8Array with Bun-portable .toString)
//   - process.stdout.write for result emission (Bun-portable; rusty-bun-
//     host's bootRequire path doesn't capture stdout, so a fallback to
//     globalThis.__asyncResult is kept for the host-internal test path)

const fs = require("node:fs");
const { makeRecordStream } = require("./lib/source");
const { makeFilterTransform, makeMapTransform } = require("./lib/transform");
const { makeFileSink } = require("./lib/sink");

async function runPipeline(records, outPath, signal) {
    if (signal && signal.aborted) {
        throw new Error("aborted before start");
    }
    const source = makeRecordStream(records, signal);
    const evenOnly = makeFilterTransform((r) => r.value % 2 === 0);
    const doubled = makeMapTransform((r) => ({ id: r.id, value: r.value * 2 }));
    const sink = makeFileSink(outPath);

    const sourceReader = source.getReader();
    const evenWriter = evenOnly.writable.getWriter();
    const doubledWriter = doubled.writable.getWriter();
    const sinkWriter = sink.stream.getWriter();

    (async () => {
        while (true) {
            if (signal && signal.aborted) {
                await evenWriter.abort("aborted");
                return;
            }
            const { value, done } = await sourceReader.read();
            if (done) { await evenWriter.close(); return; }
            await evenWriter.write(value);
        }
    })();

    (async () => {
        const r = evenOnly.readable.getReader();
        while (true) {
            const { value, done } = await r.read();
            if (done) { await doubledWriter.close(); return; }
            await doubledWriter.write(value);
        }
    })();

    const r = doubled.readable.getReader();
    while (true) {
        const { value, done } = await r.read();
        if (done) { await sinkWriter.close(); break; }
        await sinkWriter.write(value);
    }

    return sink.getCount();
}

async function selfTest() {
    const results = [];
    const outPath = "/tmp/rusty-bun-stream-processor-out.json";

    const records = [
        { id: "a", value: 1 },
        { id: "b", value: 2 },
        { id: "c", value: 3 },
        { id: "d", value: 4 },
        { id: "e", value: 6 },
    ];
    const count = await runPipeline(records, outPath, null);
    results.push(["pipeline-count", count === 3]);

    // Read written output. Bun-portable signature: readFileSync(path, "utf8").
    const written = fs.readFileSync(outPath, "utf8");
    const lines = written.trim().split("\n").map((l) => JSON.parse(l));
    results.push(["pipeline-output", lines.length === 3 &&
        lines[0].value === 4 && lines[1].value === 8 && lines[2].value === 12]);

    const ac = new AbortController();
    ac.abort();
    let caught = null;
    try {
        await runPipeline(records, outPath + ".abort", ac.signal);
    } catch (e) {
        caught = e;
    }
    results.push(["abort-honored", caught !== null]);

    let deferredCount = 0;
    setTimeout(() => { deferredCount = count; }, 0);
    await new Promise((resolve) => setTimeout(resolve, 0));
    results.push(["timer-deferred", deferredCount === 3]);

    const h = new Headers();
    h.set("x-record-count", String(count));
    h.set("content-type", "application/x-ndjson");
    results.push(["headers", h.get("x-record-count") === "3" &&
        h.get("content-type") === "application/x-ndjson"]);

    // Bun-portable Buffer instance API: .toString("hex").
    const summary = "count=" + count + ",bytes=" + written.length;
    const hex = Buffer.from(summary).toString("hex");
    results.push(["buffer-hex", hex.length === summary.length * 2 && /^[0-9a-f]+$/.test(hex)]);

    const sinkUrl = new URL("/ingest", "https://example.com:8443/v1/");
    sinkUrl.searchParams.set("count", String(count));
    sinkUrl.searchParams.set("checksum", hex.slice(0, 8));
    results.push(["url-build", sinkUrl.href.startsWith("https://example.com:8443/ingest?count=3&checksum=")]);

    fs.unlinkSync(outPath);
    results.push(["cleanup", !fs.existsSync(outPath)]);

    return results;
}

selfTest().then((results) => {
    const passed = results.filter(([_, ok]) => ok).length;
    const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
    const summary = passed + "/" + results.length +
        (failed.length > 0 ? " failed: " + failed.join(",") : "");
    process.stdout.write(summary + "\n");
}).catch((e) => {
    const msg = String(e && e.message ? e.message : e);
    process.stderr.write("error: " + msg + "\n");
    process.exit(1);
});
