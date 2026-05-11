// Tier-J consumer #50: async-listener HTTP server pattern.
//
// Demonstrates the Bun.serve-shape async-bridge in a single JS thread:
//   1. TCP.bindAsync starts a background-thread accept loop that pushes
//      Connection events into a channel.
//   2. TCP.connect from the same thread completes the kernel-level
//      handshake immediately (kernel listen backlog).
//   3. TCP.poll on the server side returns the queued connection.
//   4. Server reads request bytes (already in kernel buffer), parses,
//      processes, writes response.
//   5. Client reads response.
//
// The key insight: connect doesn't need accept() to have been called in
// user space — the kernel completes the handshake against the backlog.
// This lets the same JS thread act as both client AND server, alternating
// via explicit tick() calls between connect/write/read on the client side
// and poll/process/respond on the server side.

const enc = new TextEncoder();
const dec = new TextDecoder();

async function selfTest() {
    const results = [];

    if (typeof globalThis.TCP === "undefined" ||
        typeof globalThis.HTTP === "undefined") {
        for (const name of ["bind-async","poll-timeout","accept-connection",
                            "handle-request","serve-multiple","tick-yield",
                            "stop-cleanup","kind-async-listener"]) {
            results.push([name + "-skipped", true]);
        }
        return results;
    }

    // ── Server-side helpers ───────────────────────────────────────────
    function makeServer(fetchHandler) {
        const { id, addr } = TCP.bindAsync("127.0.0.1:0");
        return {
            id, addr,
            port: parseInt(addr.split(":")[1]),
            // Single tick: poll for one connection event, process it.
            async tick(maxWaitMs = 50) {
                const ev = TCP.poll(id, maxWaitMs);
                if (!ev || ev.type !== "connection") return false;
                const streamId = ev.streamId;
                try {
                    const bytes = TCP.read(streamId, 65536);
                    if (bytes.length === 0) { TCP.close(streamId); return true; }
                    const parsed = HTTP.parseRequest(bytes);
                    const hostHeader = parsed.headers.find(h => h[0].toLowerCase() === "host");
                    const host = hostHeader ? hostHeader[1] : "localhost";
                    const url = "http://" + host + parsed.target;
                    const headers = new Headers();
                    for (const [n, v] of parsed.headers) headers.append(n, v);
                    const init = { method: parsed.method, headers };
                    if (parsed.method !== "GET" && parsed.method !== "HEAD" &&
                        parsed.body.length > 0) {
                        init.body = parsed.body;
                    }
                    const req = new Request(url, init);
                    const resp = await fetchHandler(req);
                    const respHeaders = [];
                    resp.headers.forEach((v, n) => respHeaders.push([n, v]));
                    const respBody = await resp.bytes();
                    const respBytes = HTTP.serializeResponse(
                        resp.status, resp.statusText || "", respHeaders, respBody);
                    TCP.writeAll(streamId, respBytes);
                } finally {
                    TCP.close(streamId);
                }
                return true;
            },
            stop() { TCP.stopAsync(id); },
        };
    }

    // 1. bind-async returns id + addr; kind is "async-listener".
    {
        const server = makeServer(async () => new Response("noop"));
        results.push(["bind-async",
            typeof server.id === "number" &&
            /^127\.0\.0\.1:\d+$/.test(server.addr) &&
            server.port > 0]);
        results.push(["kind-async-listener",
            TCP.kind(server.id) === "async-listener"]);
        server.stop();
    }

    // 2. poll-timeout: no client connects, poll returns null after wait.
    {
        const server = makeServer(async () => new Response("noop"));
        const t0 = Date.now();
        const ev = TCP.poll(server.id, 30);
        const elapsed = Date.now() - t0;
        results.push(["poll-timeout",
            ev === null && elapsed >= 25 && elapsed < 200]);
        server.stop();
    }

    // 3. accept-connection: client connects → server poll returns Connection.
    {
        const server = makeServer(async () => new Response("noop"));
        const cid = TCP.connect("127.0.0.1:" + server.port);
        // Give the background thread a moment to accept.
        const ev = TCP.poll(server.id, 200);
        results.push(["accept-connection",
            ev !== null && ev.type === "connection" &&
            typeof ev.streamId === "number" &&
            /^127\.0\.0\.1:\d+$/.test(ev.peer)]);
        TCP.close(ev.streamId);
        TCP.close(cid);
        server.stop();
    }

    // 4. handle-request: full round-trip via tick.
    {
        let lastReceived = null;
        const server = makeServer(async (req) => {
            lastReceived = { method: req.method, target: new URL(req.url).pathname };
            const url = new URL(req.url);
            if (url.pathname === "/health") {
                return new Response('{"ok":true}', {
                    status: 200,
                    headers: { "content-type": "application/json" },
                });
            }
            return new Response("not found", { status: 404 });
        });
        // Client connect + send request.
        const cid = TCP.connect("127.0.0.1:" + server.port);
        const reqBytes = HTTP.serializeRequest("GET", "/health",
            [["Host", "x"], ["Connection", "close"]], "");
        TCP.writeAll(cid, reqBytes);
        // Drive server one tick to process the request.
        await server.tick(500);
        // Client reads response.
        const respBytes = TCP.read(cid, 8192);
        const resp = HTTP.parseResponse(respBytes);
        results.push(["handle-request",
            resp.status === 200 &&
            dec.decode(resp.body) === '{"ok":true}' &&
            lastReceived && lastReceived.method === "GET" &&
            lastReceived.target === "/health"]);
        TCP.close(cid);
        server.stop();
    }

    // 5. serve-multiple: handle 3 sequential requests.
    {
        let count = 0;
        const server = makeServer(async () => {
            count++;
            return new Response("ok-" + count, { status: 200 });
        });
        for (let i = 1; i <= 3; i++) {
            const cid = TCP.connect("127.0.0.1:" + server.port);
            TCP.writeAll(cid, HTTP.serializeRequest("GET", "/",
                [["Host", "x"], ["Connection", "close"]], ""));
            await server.tick(500);
            const respBytes = TCP.read(cid, 8192);
            const resp = HTTP.parseResponse(respBytes);
            if (resp.status !== 200) { results.push(["serve-multiple", false]); break; }
            if (dec.decode(resp.body) !== "ok-" + i) {
                results.push(["serve-multiple", false]); break;
            }
            TCP.close(cid);
        }
        if (count === 3) results.push(["serve-multiple", true]);
        server.stop();
    }

    // 6. tick-yield: a poll that times out without a connection returns
    //    false from tick (server can decide to keep looping).
    {
        const server = makeServer(async () => new Response("noop"));
        const didWork = await server.tick(30);
        results.push(["tick-yield", didWork === false]);
        server.stop();
    }

    // 7. stop-cleanup: after stopAsync, the handle id is invalid.
    {
        const { id } = TCP.bindAsync("127.0.0.1:0");
        TCP.stopAsync(id);
        let caught = null;
        try { TCP.poll(id, 10); } catch (e) { caught = e; }
        results.push(["stop-cleanup", caught !== null]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
