// Tier-J consumer #30: HKDF (RFC 5869) deriveBits over SHA-1/256/384/512.
//
// Validates E.8 extension to HKDF — the Extract-and-Expand KDF used by
// JOSE A*GCMKW content-key derivation, OAuth2 PoP, TLS 1.3 key schedule,
// Noise Protocol handshake state. Test vectors from RFC 5869 Appendix A.

function hexToBytes(hex) {
    const out = new Uint8Array(hex.length / 2);
    for (let i = 0; i < out.length; i++) {
        out[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return out;
}
function bytesToHex(buf) {
    const arr = buf instanceof ArrayBuffer ? new Uint8Array(buf) : buf;
    let hex = "";
    for (let i = 0; i < arr.length; i++) {
        hex += arr[i].toString(16).padStart(2, "0");
    }
    return hex;
}

async function importHkdf(ikm) {
    return crypto.subtle.importKey("raw", ikm, { name: "HKDF" }, false, ["deriveBits"]);
}

async function selfTest() {
    const results = [];

    // ── RFC 5869 Appendix A test cases ────────────────────────────────

    // A.1 (SHA-256): basic test.
    {
        const key = await importHkdf(hexToBytes("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"));
        const out = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256",
              salt: hexToBytes("000102030405060708090a0b0c"),
              info: hexToBytes("f0f1f2f3f4f5f6f7f8f9") },
            key, 42 * 8,
        );
        results.push(["rfc5869-a1-sha256",
            out instanceof ArrayBuffer && out.byteLength === 42 &&
            bytesToHex(out) ===
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"]);
    }

    // A.2 (SHA-256): longer inputs/outputs.
    {
        const ikm = hexToBytes(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f" +
            "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f" +
            "404142434445464748494a4b4c4d4e4f");
        const salt = hexToBytes(
            "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f" +
            "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f" +
            "a0a1a2a3a4a5a6a7a8a9aaabacadaeaf");
        const info = hexToBytes(
            "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf" +
            "d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef" +
            "f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
        const key = await importHkdf(ikm);
        const out = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt, info }, key, 82 * 8);
        results.push(["rfc5869-a2-sha256-longer",
            bytesToHex(out) ===
            "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c" +
            "59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71" +
            "cc30c58179ec3e87c14c01d5c1f3434f1d87"]);
    }

    // A.3 (SHA-256): empty salt + empty info.
    {
        const key = await importHkdf(hexToBytes("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"));
        const out = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: new Uint8Array(0), info: new Uint8Array(0) },
            key, 42 * 8);
        results.push(["rfc5869-a3-empty-salt-info",
            bytesToHex(out) ===
            "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"]);
    }

    // A.4 (SHA-1): basic test.
    {
        const key = await importHkdf(hexToBytes("0b0b0b0b0b0b0b0b0b0b0b"));
        const out = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-1",
              salt: hexToBytes("000102030405060708090a0b0c"),
              info: hexToBytes("f0f1f2f3f4f5f6f7f8f9") },
            key, 42 * 8);
        results.push(["rfc5869-a4-sha1",
            bytesToHex(out) ===
            "085a01ea1b10f36933068b56efa5ad81a4f14b822f5b091568a9cdd4f155fda2c22e422478d305f3f896"]);
    }

    // ── Structural properties ─────────────────────────────────────────

    // 5. Hash isolation: SHA-256 vs SHA-512 with same inputs produce
    //    distinct outputs.
    {
        const enc = new TextEncoder();
        const k1 = await importHkdf(enc.encode("ikm"));
        const k2 = await importHkdf(enc.encode("ikm"));
        const a = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info") },
            k1, 256);
        const b = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-512", salt: enc.encode("salt"), info: enc.encode("info") },
            k2, 256);
        results.push(["hash-isolation",
            a.byteLength === 32 && b.byteLength === 32 &&
            bytesToHex(a) !== bytesToHex(b)]);
    }

    // 6. info sensitivity.
    {
        const enc = new TextEncoder();
        const k1 = await importHkdf(enc.encode("ikm"));
        const k2 = await importHkdf(enc.encode("ikm"));
        const a = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info-A") },
            k1, 256);
        const b = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info-B") },
            k2, 256);
        results.push(["info-sensitive", bytesToHex(a) !== bytesToHex(b)]);
    }

    // 7. Length-truncation prefix property: derive 32 then 16 — first
    //    16 bytes of the 32-byte output match the 16-byte output.
    {
        const enc = new TextEncoder();
        const k1 = await importHkdf(enc.encode("ikm"));
        const k2 = await importHkdf(enc.encode("ikm"));
        const full = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info") },
            k1, 256);
        const half = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info") },
            k2, 128);
        const fullHex = bytesToHex(full);
        const halfHex = bytesToHex(half);
        results.push(["length-truncation-prefix",
            full.byteLength === 32 && half.byteLength === 16 &&
            fullHex.startsWith(halfHex)]);
    }

    // 8. Multi-block expansion: derive > HashLen exercises the T(i)
    //    chain. SHA-256 has HashLen=32; derive 100 bytes.
    {
        const enc = new TextEncoder();
        const key = await importHkdf(enc.encode("ikm"));
        const out = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info") },
            key, 800);
        results.push(["multi-block-expansion", out.byteLength === 100]);
    }

    // 9. Algorithm name normalization: "hkdf" / "HKDF" both accepted.
    {
        const enc = new TextEncoder();
        const k1 = await importHkdf(enc.encode("ikm"));
        const k2 = await importHkdf(enc.encode("ikm"));
        const a = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-256", salt: enc.encode("salt"), info: enc.encode("info") },
            k1, 256);
        const b = await crypto.subtle.deriveBits(
            { name: "hkdf", hash: "sha-256", salt: enc.encode("salt"), info: enc.encode("info") },
            k2, 256);
        results.push(["alg-name-normalized", bytesToHex(a) === bytesToHex(b)]);
    }

    // 10. SHA-384 round-trip determinism.
    {
        const enc = new TextEncoder();
        const k1 = await importHkdf(enc.encode("ikm"));
        const k2 = await importHkdf(enc.encode("ikm"));
        const a = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-384", salt: enc.encode("salt"), info: enc.encode("info") },
            k1, 384);
        const b = await crypto.subtle.deriveBits(
            { name: "HKDF", hash: "SHA-384", salt: enc.encode("salt"), info: enc.encode("info") },
            k2, 384);
        results.push(["sha384-deterministic",
            a.byteLength === 48 && bytesToHex(a) === bytesToHex(b)]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
