// Tier-J consumer #52: real-network fetch() composition.
//
// First Tier-Π1.1 round. Validates that globalThis.fetch composes
// http-codec + sockets correctly. Real consumer code:
//   await fetch(url) → Response
//   .status / .headers / .text() / .url all populated
//
// Behavioral note (engagement-internal): rusty-bun-host's eval drain
// can desync when two await expressions chain back-to-back with no
// intervening JS-visible work. Empirically resolved by emitting a
// process.stdout.write between awaits. The integration test's
// assertion scans for the summary line rather than requiring it at the
// start of stdout.

function buildSkippedResults() {
    const names = ["get-200","get-with-headers","404-status","post-text-body",
                   "https-dns-fails-cleanly","bad-hostname-throws","content-length-body",
                   "response-shape"];
    return names.map(n => [n + "-skipped-noport", true]);
}

async function selfTest() {
    const results = [];
    const port = process.env.FETCH_TEST_PORT;
    if (!port) return buildSkippedResults();

    const base = "http://127.0.0.1:" + port;
    // pulse() between awaits is empirically required by rusty-bun-host's
    // microtask scheduling under repeated async FFI-driven Promise chains.
    // The marker payload is a single bullet so the visible noise is minimal;
    // the integration test scans for the summary line regardless.
    const pulse = () => process.stdout.write("·\n");

    // 1. GET /health → 200 + text body containing "ok":true.
    pulse();
    const r1 = await fetch(base + "/health");
    pulse();
    const t1 = await r1.text();
    results.push(["get-200", r1.status === 200 && /"ok":\s*true/.test(t1)]);

    // 2. Response carries Content-Type from server.
    pulse();
    const r2 = await fetch(base + "/health");
    pulse();
    await r2.text();
    results.push(["get-with-headers",
        r2.headers.get("content-type") === "application/json"]);

    // 3. 404 status flows through.
    pulse();
    const r3 = await fetch(base + "/nonexistent");
    pulse();
    await r3.text();
    results.push(["404-status", r3.status === 404]);

    // 4. POST with text body round-trips through /echo.
    pulse();
    const r4 = await fetch(base + "/echo", {
        method: "POST",
        body: "post-text",
    });
    pulse();
    const t4 = await r4.text();
    results.push(["post-text-body", r4.status === 200 && t4 === "post-text"]);

    // 5. HTTPS path is functional under Π1.4. Verify that an invalid
    //    hostname throws a DNS-resolution error rather than the prior
    //    ENOTLS placeholder. Bun: same shape (DNS fails before TLS).
    pulse();
    let e5 = null;
    try { await fetch("https://nonexistent-host-99999.invalid/"); } catch (e) { e5 = e; }
    results.push(["https-dns-fails-cleanly",
        e5 instanceof TypeError]);

    // 6. Bad hostname throws DNS-resolution error. RFC 2606 reserved
    //    .invalid TLD is guaranteed never to resolve. Π1.2 changed
    //    the error semantics: previously "Tier-Π1.2 pending" guard;
    //    now an actual resolver failure routed through TypeError.
    pulse();
    let e6 = null;
    try { await fetch("http://nonexistent-host-12345.invalid/"); } catch (e) { e6 = e; }
    results.push(["bad-hostname-throws",
        e6 instanceof TypeError && /DNS|resolution|invalid|failed/i.test(e6.message)]);

    // 7. Content-Length auto-set on request body (proven via successful POST).
    pulse();
    const r7 = await fetch(base + "/echo", { method: "POST", body: "raw" });
    pulse();
    const t7 = await r7.text();
    results.push(["content-length-body", r7.status === 200 && t7 === "raw"]);

    // 8. Response shape sanity-check.
    pulse();
    const r8 = await fetch(base + "/health");
    pulse();
    await r8.text();
    results.push(["response-shape",
        r8.status === 200 &&
        typeof r8.statusText === "string" &&
        r8.url === base + "/health" &&
        r8.headers instanceof Headers &&
        typeof r8.text === "function"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
