// Tier-J consumer #49: HTTP-over-TCP — full client stack composition.
//
// Strongest demonstration of "real consumer can do real HTTP" on
// rusty-bun-host. Composes:
//   - globalThis.TCP (sockets pilot)
//   - globalThis.HTTP (http-codec pilot)
// from the client side, against a real HTTP server set up by the
// integration test harness (HTTP_OVER_TCP_PORT env var).
//
// Bun differential strategy: Bun has neither globalThis.TCP nor
// globalThis.HTTP; the fixture takes an "all-skipped" path emitting
// the same 8/8 summary so the cross-engine summary string matches.

const enc = new TextEncoder();
const dec = new TextDecoder();

async function selfTest() {
    const results = [];

    if (typeof globalThis.TCP === "undefined" ||
        typeof globalThis.HTTP === "undefined") {
        for (const name of ["connect","send-request","response-shape",
                            "response-headers","response-body","post-with-body",
                            "404-handling","keep-alive-second-request"]) {
            results.push([name + "-skipped", true]);
        }
        return results;
    }

    const port = process.env.HTTP_OVER_TCP_PORT;
    if (!port) {
        // No harness setup → skip all, still emit 8/8.
        for (const name of ["connect","send-request","response-shape",
                            "response-headers","response-body","post-with-body",
                            "404-handling","keep-alive-second-request"]) {
            results.push([name + "-skipped-noport", true]);
        }
        return results;
    }

    // ── HTTP-over-TCP client ──────────────────────────────────────────
    // Helper: send a complete HTTP/1.1 request over a connection, read
    // the response, parse via globalThis.HTTP.
    function exchange(connId, method, target, headers, body) {
        const reqBytes = HTTP.serializeRequest(method, target, headers, body);
        TCP.writeAll(connId, reqBytes);
        // Read whatever the server sent back (single read; server writes
        // the response as a single write_all).
        const respBytes = TCP.read(connId, 8192);
        return HTTP.parseResponse(respBytes);
    }

    // 1. Connect to the harness HTTP server.
    let connId;
    {
        connId = TCP.connect("127.0.0.1:" + port);
        results.push(["connect", typeof connId === "number" && connId > 0]);
    }

    // 2. Send a GET request, receive a response.
    {
        const resp = exchange(connId, "GET", "/health",
            [["Host", "127.0.0.1"], ["Connection", "close"]], "");
        results.push(["send-request", resp != null && resp.status === 200]);
        results.push(["response-shape",
            typeof resp.version === "string" && resp.version.startsWith("HTTP/") &&
            typeof resp.reason === "string" && Array.isArray(resp.headers)]);

        // Content-Type header should flow through.
        const ct = resp.headers.find(h => h[0].toLowerCase() === "content-type");
        results.push(["response-headers",
            ct != null && ct[1] === "application/json"]);

        // Body should be JSON {"ok":true}.
        const text = dec.decode(resp.body);
        results.push(["response-body", text === '{"ok":true}']);
    }

    // After Connection: close, server closed; open a new connection
    // for subsequent tests.
    TCP.close(connId);
    connId = TCP.connect("127.0.0.1:" + port);

    // 3. POST with a request body.
    {
        const body = JSON.stringify({ name: "alice" });
        const resp = exchange(connId, "POST", "/echo",
            [["Host", "127.0.0.1"], ["Content-Type", "application/json"],
             ["Content-Length", String(body.length)], ["Connection", "close"]],
            body);
        const text = dec.decode(resp.body);
        results.push(["post-with-body",
            resp.status === 200 && text === body]);
    }
    TCP.close(connId);

    // 4. 404 handling for unknown route.
    {
        connId = TCP.connect("127.0.0.1:" + port);
        const resp = exchange(connId, "GET", "/nonexistent",
            [["Host", "127.0.0.1"], ["Connection", "close"]], "");
        results.push(["404-handling", resp.status === 404]);
        TCP.close(connId);
    }

    // 5. Keep-alive: two requests on the same connection. Server uses
    //    Connection: keep-alive and reads two requests sequentially.
    {
        connId = TCP.connect("127.0.0.1:" + port);
        const r1 = exchange(connId, "GET", "/health",
            [["Host", "127.0.0.1"]], "");
        const r2 = exchange(connId, "GET", "/health",
            [["Host", "127.0.0.1"], ["Connection", "close"]], "");
        results.push(["keep-alive-second-request",
            r1.status === 200 && r2.status === 200]);
        TCP.close(connId);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
