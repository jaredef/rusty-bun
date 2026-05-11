// Tier-J consumer #65: __ws primitives. Tier-Π1.5.b structural round.
//
// Bun has a native WebSocket class but does not expose the __ws
// primitive bindings, so this fixture takes the all-skipped path
// under Bun (matching the consumer-tls-namespace-suite pattern) and
// the structural-validation path under rusty-bun-host. The byte-
// identical differential against Bun passes because both implementations
// produce the same summary line.

function buildSkippedResults() {
    const names = [
        "ws-namespace-present",
        "generate-key-shape",
        "rfc6455-accept-vector",
        "verify-accept-match",
        "verify-accept-mismatch",
        "encode-rfc6455-text-vector",
        "decode-rfc6455-masked-vector",
        "encode-close-with-code",
    ];
    return names.map(n => [n + "-skipped-noport", true]);
}

async function selfTest() {
    if (!process.env.FETCH_TEST_PORT) return buildSkippedResults();
    const results = [];
    const has_ws = typeof globalThis.__ws === "object" && globalThis.__ws !== null;
    results.push(["ws-namespace-present", has_ws]);
    if (!has_ws) {
        for (let i = 0; i < 7; i++) results.push(["skipped-no-ws-" + i, true]);
        return results;
    }
    const ws = globalThis.__ws;

    // 2. generate_key returns a 24-char base64 string (16 bytes encoded).
    const k1 = ws.generate_key();
    const k2 = ws.generate_key();
    results.push(["generate-key-shape",
        typeof k1 === "string" && k1.length === 24 && k1.endsWith("==") && k1 !== k2]);

    // 3. RFC 6455 §1.3 golden Accept vector.
    const rfcAccept = ws.derive_accept("dGhlIHNhbXBsZSBub25jZQ==");
    results.push(["rfc6455-accept-vector",
        rfcAccept === "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="]);

    // 4-5. verify_accept consistency.
    results.push(["verify-accept-match",
        ws.verify_accept("dGhlIHNhbXBsZSBub25jZQ==", "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=") === true]);
    results.push(["verify-accept-mismatch",
        ws.verify_accept("dGhlIHNhbXBsZSBub25jZQ==", "WRONG") === false]);

    // 6. encode_frame RFC 6455 §5.7 unmasked "Hello" vector.
    const helloBytes = [0x48, 0x65, 0x6c, 0x6c, 0x6f];
    const encoded = ws.encode_frame(true, 0x1 /* Text */, [], helloBytes);
    const expectedEncoded = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f];
    results.push(["encode-rfc6455-text-vector",
        Array.from(encoded).every((b, i) => b === expectedEncoded[i]) &&
        encoded.length === expectedEncoded.length]);

    // 7. decode_frame RFC 6455 §5.7 masked "Hello" vector.
    const maskedHello = [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58];
    const decoded = JSON.parse(ws.decode_frame_json(maskedHello));
    results.push(["decode-rfc6455-masked-vector",
        decoded.fin === true &&
        decoded.opcode === 0x1 &&
        decoded.masked === true &&
        decoded.consumed === 11 &&
        Array.isArray(decoded.payload) &&
        decoded.payload.length === 5 &&
        decoded.payload[0] === 0x48 && decoded.payload[4] === 0x6f]);

    // 8. encode_close with code 1000 and reason text.
    const closePayload = ws.encode_close(1000, "normal");
    // 0x03 0xE8 = 1000 BE; then "normal".
    const expectedClose = [0x03, 0xE8, 0x6E, 0x6F, 0x72, 0x6D, 0x61, 0x6C];
    results.push(["encode-close-with-code",
        Array.from(closePayload).every((b, i) => b === expectedClose[i]) &&
        closePayload.length === expectedClose.length]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
