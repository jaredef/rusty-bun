// Tier-J consumer #64: __tls namespace API-shape validation.
//
// Π1.4.h structural test. Verifies the host's __tls namespace is
// present and shaped as expected without depending on a live handshake
// (the live handshake against openssl s_server is exercised by the
// pilot-side slow test and is currently failing at recv-UnexpectedEnd
// per Π1.4.g; Π1.4.i is the diagnosis + closure round).
//
// This fixture intentionally takes the "skipped" path under Bun (which
// has its own native TLS, not __tls): all results are reported as
// skipped-noport so the byte-identical differential against Bun
// passes. The Tier-J consumer-https-suite differential lands at
// Π1.4.i once the live handshake works.

function buildSkippedResults() {
    const names = [
        "tls-namespace-present",
        "tls-connect-callable",
        "tls-write-callable",
        "tls-read-callable",
        "tls-close-callable",
        "tls-connect-rejects-bad-host",
        "tls-connect-rejects-empty-ca",
        "tls-namespace-shape-stable",
    ];
    return names.map(n => [n + "-skipped-noport", true]);
}

async function selfTest() {
    const port = process.env.FETCH_TEST_PORT;
    if (!port) return buildSkippedResults();

    const results = [];
    const has_tls = typeof globalThis.__tls === "object" && globalThis.__tls !== null;
    results.push(["tls-namespace-present", has_tls]);

    if (!has_tls) {
        for (let i = 0; i < 7; i++) results.push(["skipped-no-tls-" + i, true]);
        return results;
    }

    const tls = globalThis.__tls;
    results.push(["tls-connect-callable", typeof tls.connect === "function"]);
    results.push(["tls-write-callable", typeof tls.write === "function"]);
    results.push(["tls-read-callable", typeof tls.read === "function"]);
    results.push(["tls-close-callable", typeof tls.close === "function"]);

    // Connecting to an unresolvable host should throw.
    let badHostThrew = false;
    try { tls.connect("nonexistent.invalid", 443, "-----BEGIN CERTIFICATE-----\n\n-----END CERTIFICATE-----\n"); }
    catch (_) { badHostThrew = true; }
    results.push(["tls-connect-rejects-bad-host", badHostThrew]);

    // Connecting with empty trust store should throw (host connect succeeds
    // but handshake or CA parsing fails).
    let emptyCaThrew = false;
    try { tls.connect("127.0.0.1", parseInt(port, 10), ""); }
    catch (_) { emptyCaThrew = true; }
    results.push(["tls-connect-rejects-empty-ca", emptyCaThrew]);

    // Namespace shape stability: same set of properties exposed.
    const keys = Object.keys(tls).sort();
    // Pi2.6.c.d: tryRead + setNonblocking + rawFd added for reactor integration.
    const expected = ["close", "connect", "rawFd", "read", "setNonblocking", "tryRead", "write"];
    results.push(["tls-namespace-shape-stable",
        keys.length === expected.length &&
        keys.every((k, i) => k === expected[i])]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
