// Tier-J consumer #2 (CJS): stream-processing pipeline.
//
// Exercises orthogonal pilots from Tier-J #1:
//   - CJS module loading (require + module.exports across boundaries)
//   - ReadableStream / TransformStream / WritableStream composition
//   - AbortController / AbortSignal coordinating cancellation
//   - setTimeout for deferred work + Promise chains
//   - fs.writeFileSync / readFileSyncUtf8 for I/O sink
//   - Headers building from a derived count
//   - Buffer for hex encoding of a digest-like summary

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

    // Manually pipe source → evenOnly → doubled → sink (pipeTo deferred).
    const sourceReader = source.getReader();
    const evenWriter = evenOnly.writable.getWriter();
    const doubledWriter = doubled.writable.getWriter();
    const sinkWriter = sink.stream.getWriter();

    // source → evenOnly.writable
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

    // evenOnly.readable → doubled.writable
    (async () => {
        const r = evenOnly.readable.getReader();
        while (true) {
            const { value, done } = await r.read();
            if (done) { await doubledWriter.close(); return; }
            await doubledWriter.write(value);
        }
    })();

    // doubled.readable → sink
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

    // 1. Pipeline runs end-to-end and writes correct count.
    const records = [
        { id: "a", value: 1 },
        { id: "b", value: 2 },
        { id: "c", value: 3 },
        { id: "d", value: 4 },
        { id: "e", value: 6 },
    ];
    const count = await runPipeline(records, outPath, null);
    results.push(["pipeline-count", count === 3]);

    // 2. File contains the doubled values.
    const written = fs.readFileSyncUtf8(outPath);
    const lines = written.trim().split("\n").map((l) => JSON.parse(l));
    results.push(["pipeline-output", lines.length === 3 &&
        lines[0].value === 4 && lines[1].value === 8 && lines[2].value === 12]);

    // 3. AbortController halts at construction (signal already aborted).
    const ac = new AbortController();
    ac.abort();
    let caught = null;
    try {
        await runPipeline(records, outPath + ".abort", ac.signal);
    } catch (e) {
        caught = e;
    }
    results.push(["abort-honored", caught !== null]);

    // 4. setTimeout deferred work integrates with the pipeline.
    let deferredCount = 0;
    setTimeout(() => { deferredCount = count; }, 0);
    await new Promise((resolve) => setTimeout(resolve, 0));
    results.push(["timer-deferred", deferredCount === 3]);

    // 5. Headers built from result, using the wired pilot.
    const h = new Headers();
    h.set("x-record-count", String(count));
    h.set("content-type", "application/x-ndjson");
    results.push(["headers", h.get("x-record-count") === "3" &&
        h.get("content-type") === "application/x-ndjson"]);

    // 6. Buffer hex-encoding of a digest-shaped summary.
    const summary = "count=" + count + ",bytes=" + written.length;
    const hex = Buffer.encodeHex(Buffer.from(summary));
    results.push(["buffer-hex", hex.length === summary.length * 2 && /^[0-9a-f]+$/.test(hex)]);

    // 7. URL composition for an upstream sink (typical real-world shape).
    const sinkUrl = new URL("/ingest", "https://example.com:8443/v1/");
    sinkUrl.searchParams.set("count", String(count));
    sinkUrl.searchParams.set("checksum", hex.slice(0, 8));
    results.push(["url-build", sinkUrl.href.startsWith("https://example.com:8443/ingest?count=3&checksum=")]);

    // 8. Cleanup.
    fs.unlinkSync(outPath);
    results.push(["cleanup", !fs.existsSync(outPath)]);

    return results;
}

selfTest().then((results) => {
    const passed = results.filter(([_, ok]) => ok).length;
    const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
    globalThis.__asyncResult = passed + "/" + results.length +
        (failed.length > 0 ? " failed: " + failed.join(",") : "");
}).catch((e) => {
    globalThis.__asyncError = String(e && e.message ? e.message : e);
});
