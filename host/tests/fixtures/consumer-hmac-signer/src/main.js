// Tier-J consumer #23: HMAC-SHA-256 signing via WebCrypto (M9.bis).
//
// Closes E.8 partially: crypto.subtle.importKey + sign + verify for
// HMAC-SHA-256. Real-world JWT/webhook-signature consumer shape.
// Verification path uses timing-safe equality.

function bytesToHex(buf) {
    const arr = buf instanceof ArrayBuffer ? new Uint8Array(buf) : buf;
    let hex = "";
    for (let i = 0; i < arr.length; i++) {
        hex += arr[i].toString(16).padStart(2, "0");
    }
    return hex;
}

async function selfTest() {
    const results = [];
    const enc = new TextEncoder();

    // 1. importKey with raw bytes for HMAC-SHA-256.
    const keyBytes = enc.encode("shared-secret-key");
    const key = await crypto.subtle.importKey(
        "raw",
        keyBytes,
        { name: "HMAC", hash: "SHA-256" },
        false,
        ["sign", "verify"],
    );
    results.push(["importKey-shape",
        key.type === "secret" &&
        key.algorithm.name === "HMAC" &&
        key.algorithm.hash.name === "SHA-256" &&
        key.usages.includes("sign") &&
        key.usages.includes("verify")]);

    // 2. sign(data) returns a 32-byte ArrayBuffer (SHA-256 width).
    const message = enc.encode("hello world");
    const sig = await crypto.subtle.sign("HMAC", key, message);
    results.push(["sign-arraybuffer",
        sig instanceof ArrayBuffer && sig.byteLength === 32]);

    // 3. Deterministic: same key + same message → same signature.
    const sig2 = await crypto.subtle.sign("HMAC", key, message);
    results.push(["sign-deterministic",
        bytesToHex(sig) === bytesToHex(sig2)]);

    // 4. verify(sig, data) returns true for matching pair.
    const ok = await crypto.subtle.verify("HMAC", key, sig, message);
    results.push(["verify-matching", ok === true]);

    // 5. verify rejects tampered message (single-byte flip).
    const tampered = new Uint8Array(message);
    tampered[0] ^= 0x01;
    const tamperedOk = await crypto.subtle.verify("HMAC", key, sig, tampered);
    results.push(["verify-tampered", tamperedOk === false]);

    // 6. verify rejects bad signature (zeroed).
    const zeros = new Uint8Array(32);
    const zeroOk = await crypto.subtle.verify("HMAC", key, zeros, message);
    results.push(["verify-zero-sig", zeroOk === false]);

    // 7. Different keys produce different signatures for same message.
    const otherKey = await crypto.subtle.importKey(
        "raw", enc.encode("different-key"),
        { name: "HMAC", hash: "SHA-256" }, false, ["sign", "verify"],
    );
    const otherSig = await crypto.subtle.sign("HMAC", otherKey, message);
    results.push(["different-key-different-sig",
        bytesToHex(otherSig) !== bytesToHex(sig)]);

    // 8. Signatures across keys don't cross-verify.
    const crossOk = await crypto.subtle.verify("HMAC", otherKey, sig, message);
    results.push(["cross-key-rejected", crossOk === false]);

    // 9. JWT-style canonical use: sign a canonical-JSON payload, verify.
    const payload = { sub: "user-42", iat: 0, scope: ["read", "write"] };
    const canonicalBody = JSON.stringify(payload);
    const jwtSig = await crypto.subtle.sign("HMAC", key, enc.encode(canonicalBody));
    const jwtOk = await crypto.subtle.verify("HMAC", key, jwtSig, enc.encode(canonicalBody));
    results.push(["jwt-canonical", jwtOk === true]);

    // 10. Webhook signature shape: sign body, send header, receiver verifies.
    // RFC 4231 vector confirms the algorithm; here we just check the shape.
    const bodyBytes = enc.encode('{"event":"order.created","id":"o-1"}');
    const webhookSig = await crypto.subtle.sign("HMAC", key, bodyBytes);
    const sigHex = bytesToHex(webhookSig);
    const verifiedFromHex = await crypto.subtle.verify("HMAC", key,
        Uint8Array.from(sigHex.match(/.{2}/g).map((b) => parseInt(b, 16))),
        bodyBytes);
    results.push(["webhook-hex-roundtrip",
        sigHex.length === 64 && verifiedFromHex === true]);

    // 11. RFC 4231 Test Case 1 vector — wire-level correctness.
    // Key = 0x0b * 20; Data = "Hi There";
    // HMAC = b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
    const rfcKey = new Uint8Array(20).fill(0x0b);
    const rfcImported = await crypto.subtle.importKey(
        "raw", rfcKey, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
    const rfcSig = await crypto.subtle.sign("HMAC", rfcImported, enc.encode("Hi There"));
    results.push(["rfc4231-vector",
        bytesToHex(rfcSig) === "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
