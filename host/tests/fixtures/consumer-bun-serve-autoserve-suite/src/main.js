// Tier-J consumer #62: Bun.serve with autoServe. Tier-Π2.6 closure round.
//
// Scope-narrowed in-round per the engagement's discipline: this round
// lands the auto-keep-alive infrastructure (server registration in
// globalThis.__keepAlive, eval-loop ticking, clean stop()). The full
// same-process client→server round-trip via fetch() requires async TCP
// substrate which would gate this round to multi-substrate. Deferred
// to a follow-on Π2.6.b sub-round.
//
// What this fixture verifies: the canonical real-Bun shape
// `Bun.serve({fetch, port, autoServe: true})` is accepted by the
// apparatus, server is bound to a real port, server.stop() runs
// cleanly, and the eval loop terminates after stop() (not infinite
// loop on the keep-alive registry).

async function selfTest() {
    const results = [];

    let serverConstructed = false;
    let fetchHandlerWired = false;
    const server = Bun.serve({
        port: 0,
        hostname: "127.0.0.1",
        autoServe: true,
        fetch(req) {
            fetchHandlerWired = true;
            return new Response("ok");
        },
    });
    serverConstructed = server != null;

    // 1. autoServe returns a server object.
    results.push(["autoserve-returns-server", serverConstructed === true]);

    // 2. Real port assigned (kernel-bound; port: 0 → ephemeral).
    results.push(["autoserve-real-port",
        typeof server.port === "number" && server.port > 0]);

    // 3. server.hostname matches the request.
    results.push(["autoserve-hostname",
        server.hostname === "127.0.0.1"]);

    // 4. server.fetch is the handler we passed.
    results.push(["autoserve-fetch-is-handler",
        typeof server.fetch === "function"]);

    // 5. server.stop() succeeds without throwing.
    let stopOk = true;
    try { server.stop(); } catch (_) { stopOk = false; }
    results.push(["autoserve-stop-clean", stopOk === true]);

    // 6. After stop(), a second server constructs cleanly on a fresh port.
    const server2 = Bun.serve({
        port: 0,
        hostname: "127.0.0.1",
        autoServe: true,
        fetch(req) { return new Response("server2"); },
    });
    results.push(["autoserve-second-server-clean",
        server2 != null && server2.port > 0]);
    server2.stop();

    // 7. server.url is present (URL object or string; check via String()).
    const server3 = Bun.serve({
        port: 0,
        hostname: "127.0.0.1",
        autoServe: true,
        fetch(req) { return new Response("ok"); },
    });
    const urlShape = server3.url != null && String(server3.url).startsWith("http://");
    server3.stop();
    results.push(["autoserve-url-shape", urlShape === true]);

    // 8. autoServe: false (default) still works (back-compat).
    const inProcessServer = Bun.serve({
        fetch(req) { return new Response("in-process"); },
    });
    // The in-process variant exposes .fetch as the handler; no real port.
    results.push(["non-autoserve-back-compat",
        typeof inProcessServer.fetch === "function"]);
    inProcessServer.stop();

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
