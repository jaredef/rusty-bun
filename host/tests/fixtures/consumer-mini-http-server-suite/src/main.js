// Tier-J consumer #47: end-to-end HTTP server pattern via http-codec.
//
// Bytes-in → handler → bytes-out. Strongest "real consumer can run a
// Bun.serve-shape backend on rusty-bun-host" evidence the engagement
// produces without OS sockets. Composes:
//   - http-codec (just landed at bce3a24) for wire-format parse/serialize
//   - Web-spec Request/Response/Headers (basin-stable since the early rounds)
//   - mini-http-server (new vendored library) for the dispatch flow
//
// If globalThis.HTTP is absent (Bun side), install the same JS reference
// codec used by consumer-http-codec-suite for cross-engine differential.

if (typeof globalThis.HTTP === "undefined") {
    globalThis.HTTP = installJsReferenceCodec();
}
function installJsReferenceCodec() {
    const dec = new TextDecoder();
    const enc = new TextEncoder();
    function findHeaderEnd(b) {
        for (let i = 0; i + 3 < b.length; i++)
            if (b[i]===0x0d && b[i+1]===0x0a && b[i+2]===0x0d && b[i+3]===0x0a) return i + 4;
        return -1;
    }
    function findCRLF(slice) {
        for (let i = 0; i + 1 < slice.length; i++)
            if (slice[i]===0x0d && slice[i+1]===0x0a) return i;
        return -1;
    }
    function ciGet(headers, name) {
        const l = name.toLowerCase();
        for (const [k, v] of headers) if (k.toLowerCase() === l) return v;
        return null;
    }
    function parseHeaders(slice) {
        const out = [];
        if (slice.length === 0) return out;
        for (const line of dec.decode(slice).split("\r\n")) {
            if (!line) continue;
            const idx = line.indexOf(":");
            if (idx < 0) throw new Error("bad header");
            out.push([line.slice(0, idx).trim(), line.slice(idx + 1).trim()]);
        }
        return out;
    }
    function toBytes(b) {
        if (b == null) return new Uint8Array(0);
        if (b instanceof Uint8Array) return b;
        if (b instanceof ArrayBuffer) return new Uint8Array(b);
        return enc.encode(String(b));
    }
    function decodeBody(headers, body) {
        const cl = ciGet(headers, "content-length");
        if (cl != null) {
            const n = parseInt(cl, 10);
            return body.slice(0, n);
        }
        return body;
    }
    function writeHeadersStr(headers, bodyLen) {
        let s = "", hasCL = false, hasTE = false;
        for (const [n, v] of headers) {
            if (n.toLowerCase() === "content-length") hasCL = true;
            if (n.toLowerCase() === "transfer-encoding") hasTE = true;
            s += n + ": " + v + "\r\n";
        }
        if (!hasCL && !hasTE) s += "Content-Length: " + bodyLen + "\r\n";
        return s;
    }
    return {
        parseRequest(bytes) {
            const b = toBytes(bytes);
            const hend = findHeaderEnd(b);
            if (hend < 0) throw new Error("truncated");
            const hsec = b.subarray(0, hend - 4);
            const bsec = b.subarray(hend);
            const crlf = findCRLF(hsec);
            const startLine = dec.decode(crlf >= 0 ? hsec.subarray(0, crlf) : hsec);
            const headersBytes = crlf >= 0 ? hsec.subarray(crlf + 2) : new Uint8Array(0);
            const parts = startLine.split(" ");
            return {
                method: parts[0],
                target: parts[1],
                version: parts.slice(2).join(" "),
                headers: parseHeaders(headersBytes),
                body: decodeBody(parseHeaders(headersBytes), bsec),
            };
        },
        serializeResponse(status, reason, headers, body) {
            const b = toBytes(body);
            let s = "HTTP/1.1 " + status + " " + (reason || "") + "\r\n";
            s += writeHeadersStr(headers, b.length);
            s += "\r\n";
            const head = enc.encode(s);
            const out = new Uint8Array(head.length + b.length);
            out.set(head, 0); out.set(b, head.length);
            return out;
        },
    };
}

import { MiniHttpServer } from "mini-http-server";

const enc = new TextEncoder();
const dec = new TextDecoder();

async function selfTest() {
    const results = [];

    // Build a fetch-handler that mimics a real backend.
    async function fetchHandler(req) {
        const url = new URL(req.url);
        if (url.pathname === "/health") {
            return new Response(JSON.stringify({ ok: true }), {
                status: 200,
                headers: { "content-type": "application/json" },
            });
        }
        if (url.pathname === "/echo") {
            let body = "";
            if (req.method !== "GET") {
                try { body = await req.text(); }
                catch (e) { body = "ERR:" + e.message; }
            }
            return new Response("echo:" + body, { status: 200 });
        }
        if (url.pathname === "/json" && req.method === "POST") {
            const data = await req.json();
            return new Response(JSON.stringify({ received: data }), {
                status: 201,
                headers: { "content-type": "application/json" },
            });
        }
        if (url.pathname === "/headers") {
            const auth = req.headers.get("authorization");
            return new Response(JSON.stringify({ auth }), { status: 200 });
        }
        if (url.pathname === "/throw") {
            throw new Error("intentional");
        }
        if (url.pathname === "/redirect") {
            return new Response(null, { status: 301, headers: { location: "/new" } });
        }
        return new Response("not found", { status: 404 });
    }

    const server = new MiniHttpServer({ fetch: fetchHandler });

    // 1. GET /health → 200 OK with JSON body.
    {
        const reqBytes = enc.encode("GET /health HTTP/1.1\r\nHost: example.com\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["health-200",
            text.startsWith("HTTP/1.1 200 OK\r\n") &&
            text.includes("content-type: application/json") &&
            text.endsWith('{"ok":true}')]);
    }

    // 2. POST /echo with body → echoes back.
    {
        const reqBytes = enc.encode(
            "POST /echo HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\n\r\nhello");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        const ok = text.startsWith("HTTP/1.1 200") && text.endsWith("echo:hello");
        if (!ok) process.stdout.write("DEBUG echo response: " + JSON.stringify(text) + "\n");
        results.push(["echo-post", ok]);
    }

    // 3. POST /json with JSON body → 201 Created.
    {
        const body = JSON.stringify({ name: "alice", age: 30 });
        const reqBytes = enc.encode(
            "POST /json HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\n" +
            "Content-Length: " + body.length + "\r\n\r\n" + body);
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["json-201",
            text.startsWith("HTTP/1.1 201 Created\r\n") &&
            text.includes('"received":{"name":"alice","age":30}')]);
    }

    // 4. Headers flow through Request and back.
    {
        const reqBytes = enc.encode(
            "GET /headers HTTP/1.1\r\nHost: x\r\nAuthorization: Bearer abc123\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["request-headers",
            text.endsWith('{"auth":"Bearer abc123"}')]);
    }

    // 5. Unmatched route → 404.
    {
        const reqBytes = enc.encode("GET /missing HTTP/1.1\r\nHost: x\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["unmatched-404",
            text.startsWith("HTTP/1.1 404 Not Found\r\n")]);
    }

    // 6. Throwing handler → onError → 500 default.
    {
        const reqBytes = enc.encode("GET /throw HTTP/1.1\r\nHost: x\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["handler-throws-500",
            text.startsWith("HTTP/1.1 500 Internal Server Error\r\n") &&
            text.includes("intentional")]);
    }

    // 7. Malformed request bytes → 400.
    {
        const reqBytes = enc.encode("not a valid http request");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["malformed-400",
            text.startsWith("HTTP/1.1 400 Bad Request\r\n")]);
    }

    // 8. Custom onError handler is used.
    {
        const customServer = new MiniHttpServer({
            fetch: async () => { throw new Error("oops"); },
            onError: () => new Response("custom-error", { status: 503 }),
        });
        const reqBytes = enc.encode("GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        const respBytes = await customServer.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["custom-on-error",
            text.startsWith("HTTP/1.1 503") && text.endsWith("custom-error")]);
    }

    // 9. Redirect response with empty body + Location header.
    {
        const reqBytes = enc.encode("GET /redirect HTTP/1.1\r\nHost: x\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        const text = dec.decode(respBytes);
        results.push(["redirect-301",
            text.startsWith("HTTP/1.1 301 Moved Permanently\r\n") &&
            text.includes("location: /new") &&
            text.endsWith("\r\n\r\n")]);
    }

    // 10. Full request → response round-trip via re-parsing the response.
    {
        const reqBytes = enc.encode("GET /health HTTP/1.1\r\nHost: x\r\n\r\n");
        const respBytes = await server.handle(reqBytes);
        // Re-parse the response bytes; verify structure.
        const parsed = HTTP.parseRequest != null
            ? null  // parseResponse may not be in the fallback codec; use text check.
            : null;
        const text = dec.decode(respBytes);
        results.push(["roundtrip-response-shape",
            /^HTTP\/1\.1 \d\d\d /.test(text) &&
            text.includes("\r\n\r\n")]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
