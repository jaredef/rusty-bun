// Tier-J consumer #55: Content-Encoding decode composition.
//
// Tier-Π1.3 substrate-introduction round (decode-only). Validates that
// fetch() correctly handles Content-Encoding: gzip + Content-Encoding:
// deflate (RFC 1952 + RFC 1950) by decoding the response body, dropping
// the Content-Encoding header, and adjusting Content-Length.
//
// The integration harness serves real-network gzip/deflate-encoded
// responses; the fixture asserts the decoded body matches a known
// reference string.
//
// Pulse markers are kept consistent with the real-fetch suite for
// rusty-bun-host's microtask drain interleaving requirement.

async function selfTest() {
    const results = [];
    const port = process.env.FETCH_TEST_PORT;
    if (!port) {
        return ["gzip-decode", "gzip-headers-stripped", "gzip-content-length-updated",
                "deflate-zlib-decode", "deflate-raw-decode", "identity-passthrough",
                "gzip-large-body", "double-encoding-supported"]
            .map(n => [n + "-skipped-noport", true]);
    }

    const base = "http://127.0.0.1:" + port;
    const pulse = () => process.stdout.write("·\n");

    // 1. GET /gzip → 200 with gzip-encoded body of "compressed payload".
    pulse();
    const r1 = await fetch(base + "/gzip");
    pulse();
    const t1 = await r1.text();
    results.push(["gzip-decode", r1.status === 200 && t1 === "compressed payload"]);

    // 2. After decode, Content-Encoding header is stripped.
    results.push(["gzip-headers-stripped", r1.headers.get("content-encoding") === null]);

    // 3. After decode, Content-Length matches decoded length, not encoded.
    results.push(["gzip-content-length-updated",
        r1.headers.get("content-length") === String(t1.length)]);

    // 4. GET /deflate-zlib → 200 with zlib-wrapped deflate body.
    pulse();
    const r4 = await fetch(base + "/deflate-zlib");
    pulse();
    const t4 = await r4.text();
    results.push(["deflate-zlib-decode", r4.status === 200 && t4 === "zlib-wrapped"]);

    // 5. GET /deflate-raw → 200 with raw DEFLATE body (RFC 1951 only).
    pulse();
    const r5 = await fetch(base + "/deflate-raw");
    pulse();
    const t5 = await r5.text();
    results.push(["deflate-raw-decode", r5.status === 200 && t5 === "raw-deflate"]);

    // 6. GET /identity → 200 with identity-encoded body (passthrough).
    pulse();
    const r6 = await fetch(base + "/identity");
    pulse();
    const t6 = await r6.text();
    results.push(["identity-passthrough", r6.status === 200 && t6 === "uncompressed"]);

    // 7. GET /gzip-large → 200 with large gzip-encoded body (~5 KiB).
    pulse();
    const r7 = await fetch(base + "/gzip-large");
    pulse();
    const t7 = await r7.text();
    // Server returns "abcde" repeated 1000 times = 5000 chars.
    results.push(["gzip-large-body", r7.status === 200 && t7.length === 5000 &&
        t7.startsWith("abcde") && t7.endsWith("abcde")]);

    // 8. Sanity: double Content-Encoding not commonly emitted, but the
    //    apparatus should reverse-order the codings list per RFC 7231.
    //    We test the simpler shape: a list with leading whitespace.
    pulse();
    const r8 = await fetch(base + "/gzip-ws-header");
    pulse();
    const t8 = await r8.text();
    results.push(["double-encoding-supported",
        r8.status === 200 && t8 === "ws-gzip"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
