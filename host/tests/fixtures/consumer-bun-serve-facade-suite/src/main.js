// Tier-J consumer #51: Bun.serve facade.
//
// Minimal first iteration: just verifies that the listen()+tick() pattern
// works end-to-end on rusty-bun-host, and the all-skipped path matches
// under Bun for the differential.

const enc = new TextEncoder();
const dec = new TextDecoder();

async function selfTest() {
    const results = [];

    if (typeof globalThis.TCP === "undefined" ||
        typeof globalThis.HTTP === "undefined") {
        // Bun side: declare all 5 cases passed via skip.
        for (const name of ["in-process-fetch","listen","tick-handles-request",
                            "url-property","stop-then-tick"]) {
            results.push([name + "-skipped", true]);
        }
        return results;
    }

    // 1. In-process .fetch() backward-compat: existing Bun.serve fixtures
    //    use this; verify the extension didn't break it.
    {
        const server = Bun.serve({
            port: 3000,
            fetch() { return new Response("hello"); },
        });
        const resp = await server.fetch(new Request("http://x/"));
        const text = await resp.text();
        results.push(["in-process-fetch", resp.status === 200 && text === "hello"]);
        server.stop();
    }

    // 2. listen() binds and updates server.port.
    {
        const server = Bun.serve({
            port: 0, hostname: "127.0.0.1",
            fetch() { return new Response("noop"); },
        });
        server.listen();
        results.push(["listen", server.port > 0 && server.port !== 3000]);
        server.stop();
    }

    // 3. tick() handles a real HTTP request end-to-end.
    {
        const server = Bun.serve({
            port: 0, hostname: "127.0.0.1",
            fetch(req) {
                const url = new URL(req.url);
                if (url.pathname === "/health") {
                    return new Response('{"ok":true}', {
                        status: 200,
                        headers: { "content-type": "application/json" },
                    });
                }
                return new Response("not found", { status: 404 });
            },
        });
        server.listen();
        const cid = TCP.connect("127.0.0.1:" + server.port);
        TCP.writeAll(cid, HTTP.serializeRequest("GET", "/health",
            [["Host", "x"], ["Connection", "close"]], ""));
        await server.tick(500);
        const respBytes = TCP.read(cid, 8192);
        const resp = HTTP.parseResponse(respBytes);
        results.push(["tick-handles-request",
            resp.status === 200 &&
            dec.decode(resp.body) === '{"ok":true}']);
        TCP.close(cid);
        server.stop();
    }

    // 4. server.url reflects the bound port.
    {
        const server = Bun.serve({
            port: 0, hostname: "127.0.0.1",
            fetch() { return new Response("x"); },
        });
        server.listen();
        const expected = "http://127.0.0.1:" + server.port + "/";
        results.push(["url-property", server.url === expected]);
        server.stop();
    }

    // 5. tick() after stop() returns false (no work).
    {
        const server = Bun.serve({
            port: 0, fetch: () => new Response("x"),
        });
        server.listen();
        server.stop();
        const didWork = await server.tick(10);
        results.push(["stop-then-tick", didWork === false]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
