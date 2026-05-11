// Tier-J consumer #46: HTTP/1.1 wire-format codec.
//
// First Tier-G transport-layer pilot integration test. Exercises
// parse_request / parse_response / serialize_request / serialize_response
// + chunked_encode / chunked_decode against RFC 7230 wire vectors and
// round-trip properties.
//
// Cross-engine strategy: rusty-bun-host has native globalThis.HTTP
// (FFI to the rusty-http-codec pilot). Bun does not expose its
// internal HTTP parser as JS. To make this fixture Bun-portable for
// the differential, a small JS reference implementation is installed
// when globalThis.HTTP is absent. Both engines then run the same
// test suite; the differential signal is the summary string ("N/M")
// — meaningful because two independent implementations of RFC 7230
// must produce the same parse/serialize results for the same inputs.

if (typeof globalThis.HTTP === "undefined") {
    globalThis.HTTP = installJsReferenceImpl();
}

function installJsReferenceImpl() {
    const dec = new TextDecoder();
    const enc = new TextEncoder();
    function findHeaderEnd(b) {
        for (let i = 0; i + 3 < b.length; i++) {
            if (b[i] === 0x0d && b[i+1] === 0x0a && b[i+2] === 0x0d && b[i+3] === 0x0a) return i + 4;
        }
        return -1;
    }
    function ciGet(headers, name) {
        const lower = name.toLowerCase();
        for (const [k, v] of headers) if (k.toLowerCase() === lower) return v;
        return null;
    }
    function parseHeaders(slice) {
        const out = [];
        if (slice.length === 0) return out;
        const text = dec.decode(slice);
        for (const line of text.split("\r\n")) {
            if (!line) continue;
            const idx = line.indexOf(":");
            if (idx < 0) throw new Error("bad header: " + line);
            const name = line.slice(0, idx).trim();
            const value = line.slice(idx + 1).trim();
            if (!name) throw new Error("empty header name");
            out.push([name, value]);
        }
        return out;
    }
    function chunkedDecode(bytes) {
        const out = [];
        let i = 0;
        while (i < bytes.length) {
            let j = i;
            while (j + 1 < bytes.length && !(bytes[j] === 0x0d && bytes[j+1] === 0x0a)) j++;
            const sizeLine = dec.decode(bytes.subarray(i, j));
            const sizeHex = sizeLine.split(";")[0].trim();
            const size = parseInt(sizeHex, 16);
            if (Number.isNaN(size)) throw new Error("bad chunk size: " + sizeHex);
            i = j + 2;
            if (size === 0) {
                // Skip optional trailers up to final CRLF.
                while (i + 1 < bytes.length && !(bytes[i] === 0x0d && bytes[i+1] === 0x0a)) i++;
                return new Uint8Array(out);
            }
            for (let k = 0; k < size; k++) out.push(bytes[i + k]);
            i += size;
            if (bytes[i] !== 0x0d || bytes[i+1] !== 0x0a) throw new Error("chunk not followed by CRLF");
            i += 2;
        }
        throw new Error("no zero-chunk terminator");
    }
    function decodeBody(headers, body) {
        const te = ciGet(headers, "transfer-encoding");
        if (te && te.toLowerCase().includes("chunked")) return chunkedDecode(body);
        const cl = ciGet(headers, "content-length");
        if (cl != null) {
            const n = parseInt(cl, 10);
            if (body.length < n) throw new Error("content-length mismatch");
            return body.slice(0, n);
        }
        return body;
    }
    function toBytes(b) {
        if (b instanceof Uint8Array) return b;
        if (b instanceof ArrayBuffer) return new Uint8Array(b);
        if (Array.isArray(b)) return new Uint8Array(b);
        return enc.encode(String(b));
    }
    function findCRLF(slice) {
        for (let i = 0; i + 1 < slice.length; i++)
            if (slice[i] === 0x0d && slice[i+1] === 0x0a) return i;
        return -1;
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
            if (parts.length < 3) throw new Error("bad start line");
            const method = parts[0], target = parts[1], version = parts.slice(2).join(" ");
            if (!version.startsWith("HTTP/")) throw new Error("bad version: " + version);
            const headers = parseHeaders(headersBytes);
            return { method, target, version, headers, body: decodeBody(headers, bsec) };
        },
        parseResponse(bytes) {
            const b = toBytes(bytes);
            const hend = findHeaderEnd(b);
            if (hend < 0) throw new Error("truncated");
            const hsec = b.subarray(0, hend - 4);
            const bsec = b.subarray(hend);
            const crlf = findCRLF(hsec);
            const statusLine = dec.decode(crlf >= 0 ? hsec.subarray(0, crlf) : hsec);
            const headersBytes = crlf >= 0 ? hsec.subarray(crlf + 2) : new Uint8Array(0);
            const sp1 = statusLine.indexOf(" ");
            const sp2 = statusLine.indexOf(" ", sp1 + 1);
            const version = statusLine.slice(0, sp1);
            if (!version.startsWith("HTTP/")) throw new Error("bad version");
            const statusStr = sp2 > 0 ? statusLine.slice(sp1 + 1, sp2) : statusLine.slice(sp1 + 1);
            const status = parseInt(statusStr, 10);
            if (Number.isNaN(status)) throw new Error("bad status");
            const reason = sp2 > 0 ? statusLine.slice(sp2 + 1) : "";
            const headers = parseHeaders(headersBytes);
            return { version, status, reason, headers, body: decodeBody(headers, bsec) };
        },
        serializeRequest(method, target, headers, body) {
            const b = toBytes(body);
            let s = method + " " + target + " HTTP/1.1\r\n";
            let hasCL = false, hasTE = false;
            for (const [n, v] of headers) {
                if (n.toLowerCase() === "content-length") hasCL = true;
                if (n.toLowerCase() === "transfer-encoding") hasTE = true;
                s += n + ": " + v + "\r\n";
            }
            if (!hasCL && !hasTE) s += "Content-Length: " + b.length + "\r\n";
            s += "\r\n";
            const head = enc.encode(s);
            const out = new Uint8Array(head.length + b.length);
            out.set(head, 0); out.set(b, head.length);
            return out;
        },
        serializeResponse(status, reason, headers, body) {
            const b = toBytes(body);
            let s = "HTTP/1.1 " + status + " " + (reason || "") + "\r\n";
            let hasCL = false, hasTE = false;
            for (const [n, v] of headers) {
                if (n.toLowerCase() === "content-length") hasCL = true;
                if (n.toLowerCase() === "transfer-encoding") hasTE = true;
                s += n + ": " + v + "\r\n";
            }
            if (!hasCL && !hasTE) s += "Content-Length: " + b.length + "\r\n";
            s += "\r\n";
            const head = enc.encode(s);
            const out = new Uint8Array(head.length + b.length);
            out.set(head, 0); out.set(b, head.length);
            return out;
        },
        chunkedEncode(chunks) {
            const parts = [];
            for (const c of chunks) {
                const u8 = toBytes(c);
                parts.push(enc.encode(u8.length.toString(16).toUpperCase() + "\r\n"));
                parts.push(u8);
                parts.push(enc.encode("\r\n"));
            }
            parts.push(enc.encode("0\r\n\r\n"));
            let len = 0; for (const p of parts) len += p.length;
            const out = new Uint8Array(len); let o = 0;
            for (const p of parts) { out.set(p, o); o += p.length; }
            return out;
        },
        chunkedDecode(bytes) { return chunkedDecode(toBytes(bytes)); },
    };
}

const enc = new TextEncoder();
const dec = new TextDecoder();
function bytesEq(a, b) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
}

async function selfTest() {
    const results = [];

    // 1. Parse simple GET.
    {
        const bytes = enc.encode("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
        const r = HTTP.parseRequest(bytes);
        results.push(["parse-simple-get",
            r.method === "GET" && r.target === "/" && r.version === "HTTP/1.1" &&
            r.headers.length === 1 && r.headers[0][0] === "Host" &&
            r.headers[0][1] === "example.com" &&
            r.body.length === 0]);
    }

    // 2. Parse POST with body.
    {
        const bytes = enc.encode(
            "POST /api HTTP/1.1\r\nHost: x\r\nContent-Length: 7\r\n\r\nhello!!");
        const r = HTTP.parseRequest(bytes);
        results.push(["parse-post-body",
            r.method === "POST" && r.target === "/api" &&
            dec.decode(r.body) === "hello!!"]);
    }

    // 3. Parse response.
    {
        const bytes = enc.encode("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
        const r = HTTP.parseResponse(bytes);
        results.push(["parse-response",
            r.status === 200 && r.reason === "OK" && dec.decode(r.body) === "hello"]);
    }

    // 4. Parse response with multi-word reason.
    {
        const bytes = enc.encode("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        const r = HTTP.parseResponse(bytes);
        results.push(["parse-multiword-reason",
            r.status === 404 && r.reason === "Not Found"]);
    }

    // 5. Serialize request with auto Content-Length.
    {
        const out = HTTP.serializeRequest("GET", "/", [["Host", "x"]], "");
        const expected = "GET / HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n\r\n";
        results.push(["serialize-request-auto-cl",
            dec.decode(out) === expected]);
    }

    // 6. Serialize response.
    {
        const out = HTTP.serializeResponse(200, "OK",
            [["Content-Type", "text/plain"]], "hello");
        const expected = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
        results.push(["serialize-response",
            dec.decode(out) === expected]);
    }

    // 7. Round-trip: parse → serialize → bytes equal original.
    {
        const original = enc.encode(
            "POST /api HTTP/1.1\r\nHost: example.com\r\nContent-Length: 11\r\n\r\nhello world");
        const r = HTTP.parseRequest(original);
        const back = HTTP.serializeRequest(r.method, r.target, r.headers, r.body);
        results.push(["roundtrip-request",
            bytesEq(back, original)]);
    }

    // 8. Chunked encode basic.
    {
        const out = HTTP.chunkedEncode([enc.encode("hello "), enc.encode("world")]);
        // 6 hex = "6", 5 hex = "5"
        const expected = "6\r\nhello \r\n5\r\nworld\r\n0\r\n\r\n";
        results.push(["chunked-encode",
            dec.decode(out) === expected]);
    }

    // 9. Chunked decode basic.
    {
        const bytes = enc.encode("6\r\nhello \r\n5\r\nworld\r\n0\r\n\r\n");
        const out = HTTP.chunkedDecode(bytes);
        results.push(["chunked-decode", dec.decode(out) === "hello world"]);
    }

    // 10. Chunked round-trip.
    {
        const chunks = [enc.encode("part1-"), enc.encode("part2"), enc.encode("-end")];
        const encoded = HTTP.chunkedEncode(chunks);
        const decoded = HTTP.chunkedDecode(encoded);
        results.push(["chunked-roundtrip",
            dec.decode(decoded) === "part1-part2-end"]);
    }

    // 11. Response with Transfer-Encoding: chunked body parsing.
    {
        const head = enc.encode("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");
        const body = HTTP.chunkedEncode([enc.encode("abc"), enc.encode("def")]);
        const full = new Uint8Array(head.length + body.length);
        full.set(head, 0); full.set(body, head.length);
        const r = HTTP.parseResponse(full);
        results.push(["te-chunked-parse", dec.decode(r.body) === "abcdef"]);
    }

    // 12. Headers preserve as-cased.
    {
        const bytes = enc.encode("GET / HTTP/1.1\r\nX-Custom-Header: v1\r\n\r\n");
        const r = HTTP.parseRequest(bytes);
        results.push(["header-case-preserved",
            r.headers[0][0] === "X-Custom-Header"]);
    }

    // 13. Content-Length capped reads (extra trailing bytes ignored).
    {
        const bytes = enc.encode("HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabcEXTRA");
        const r = HTTP.parseResponse(bytes);
        results.push(["content-length-cap", dec.decode(r.body) === "abc"]);
    }

    // 14. Status codes as numbers, not strings.
    {
        const bytes = enc.encode("HTTP/1.1 503 Service Unavailable\r\n\r\n");
        const r = HTTP.parseResponse(bytes);
        results.push(["status-as-number",
            typeof r.status === "number" && r.status === 503]);
    }

    // 15. Multiple headers with same name preserved as separate entries.
    {
        const bytes = enc.encode("GET / HTTP/1.1\r\nCookie: a=1\r\nCookie: b=2\r\n\r\n");
        const r = HTTP.parseRequest(bytes);
        results.push(["multi-same-header",
            r.headers.length === 2 &&
            r.headers[0][1] === "a=1" && r.headers[1][1] === "b=2"]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
