// Tier-J consumer #31: AES-CBC + AES-CTR + AES-KW.
//
// Validates E.8 extension to symmetric block-cipher modes adjacent to
// AES-GCM. CBC is still common in JOSE A128CBC-HS256 envelopes and TLS
// 1.2 record-layer; CTR is used in CTR-DRBG and IPsec ESP; KW is the
// JWE A128KW/A256KW content-key wrapping primitive.
//
// Test vectors: SP 800-38A Appendix F.2 (CBC) + F.5 (CTR) + RFC 3394 §4 (KW).

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

async function selfTest() {
    const results = [];

    // ── AES-CBC ───────────────────────────────────────────────────────

    // 1. SP 800-38A F.2.1 (AES-128-CBC) round-trip. WebCrypto AES-CBC
    //    always applies PKCS#7 padding, so a 4-block plaintext gets an
    //    extra full padding block; output is 5 blocks.
    {
        const keyBytes = hexToBytes("2b7e151628aed2a6abf7158809cf4f3c");
        const iv = hexToBytes("000102030405060708090a0b0c0d0e0f");
        const pt = hexToBytes(
            "6bc1bee22e409f96e93d7e117393172a" +
            "ae2d8a571e03ac9c9eb76fac45af8e51" +
            "30c81c46a35ce411e5fbc1191a0a52ef" +
            "f69f2445df4f9b17ad2b417be66c3710");
        const key = await crypto.subtle.importKey(
            "raw", keyBytes, { name: "AES-CBC" }, false, ["encrypt", "decrypt"]);
        const ct = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, pt);
        const dec = await crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, ct);
        results.push(["cbc-sp800-38a-f21",
            ct.byteLength === 80 &&
            bytesToHex(dec) === bytesToHex(pt)]);
    }

    // 2. CBC round-trip on arbitrary-length non-aligned plaintext.
    {
        const enc = new TextEncoder();
        const key = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "AES-CBC" }, false, ["encrypt", "decrypt"]);
        const iv = new Uint8Array(16);
        const pt = enc.encode("short message that is not block-aligned");
        const ct = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, pt);
        const dec = await crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, ct);
        results.push(["cbc-roundtrip-unaligned",
            ct.byteLength % 16 === 0 &&
            bytesToHex(dec) === bytesToHex(pt)]);
    }

    // 3. CBC tampered-ciphertext rejection (bad PKCS#7 padding).
    {
        const key = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "AES-CBC" }, false, ["encrypt", "decrypt"]);
        const iv = new Uint8Array(16);
        const pt = new TextEncoder().encode("AAAAAAAAAAAAAAAA");  // 16 bytes → 32-byte ct
        const ct = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, pt);
        const tampered = new Uint8Array(ct);
        tampered[tampered.length - 1] ^= 0x01;
        let rejected = false;
        try { await crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, tampered); }
        catch (_) { rejected = true; }
        results.push(["cbc-bad-padding-rejected", rejected]);
    }

    // ── AES-CTR ───────────────────────────────────────────────────────

    // 4. SP 800-38A F.5.1 (AES-128-CTR) wire-level vector.
    {
        const keyBytes = hexToBytes("2b7e151628aed2a6abf7158809cf4f3c");
        const counter = hexToBytes("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
        const pt = hexToBytes(
            "6bc1bee22e409f96e93d7e117393172a" +
            "ae2d8a571e03ac9c9eb76fac45af8e51" +
            "30c81c46a35ce411e5fbc1191a0a52ef" +
            "f69f2445df4f9b17ad2b417be66c3710");
        const key = await crypto.subtle.importKey(
            "raw", keyBytes, { name: "AES-CTR" }, false, ["encrypt", "decrypt"]);
        const ct = await crypto.subtle.encrypt(
            { name: "AES-CTR", counter, length: 128 }, key, pt);
        results.push(["ctr-sp800-38a-f51",
            bytesToHex(ct) ===
            "874d6191b620e3261bef6864990db6ce" +
            "9806f66b7970fdff8617187bb9fffdff" +
            "5ae4df3edbd5d35e5b4f09020db03eab" +
            "1e031dda2fbe03d1792170a0f3009cee"]);
    }

    // 5. CTR round-trip — encrypt then encrypt again (CTR is self-inverse).
    {
        const enc = new TextEncoder();
        const key = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "AES-CTR" }, false, ["encrypt", "decrypt"]);
        const counter = new Uint8Array(16);
        const pt = enc.encode("AES-CTR round-trip with arbitrary-length plaintext");
        const ct = await crypto.subtle.encrypt(
            { name: "AES-CTR", counter, length: 64 }, key, pt);
        const dec = await crypto.subtle.decrypt(
            { name: "AES-CTR", counter, length: 64 }, key, ct);
        results.push(["ctr-roundtrip",
            bytesToHex(dec) === bytesToHex(pt)]);
    }

    // 6. CTR keystream is IV-sensitive.
    {
        const enc = new TextEncoder();
        const key = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "AES-CTR" }, false, ["encrypt"]);
        const c1 = new Uint8Array(16); c1[0] = 1;
        const c2 = new Uint8Array(16); c2[0] = 2;
        const pt = enc.encode("same plaintext, different counter");
        const ct1 = await crypto.subtle.encrypt(
            { name: "AES-CTR", counter: c1, length: 64 }, key, pt);
        const ct2 = await crypto.subtle.encrypt(
            { name: "AES-CTR", counter: c2, length: 64 }, key, pt);
        results.push(["ctr-counter-sensitive", bytesToHex(ct1) !== bytesToHex(ct2)]);
    }

    // ── AES-KW ────────────────────────────────────────────────────────

    // 7. RFC 3394 §4.1: Wrap 128-bit key with 128-bit KEK.
    {
        const kekBytes = hexToBytes("000102030405060708090a0b0c0d0e0f");
        const keyData = hexToBytes("00112233445566778899aabbccddeeff");
        const kek = await crypto.subtle.importKey(
            "raw", kekBytes, { name: "AES-KW" }, false, ["wrapKey", "unwrapKey"]);
        // The to-be-wrapped key must be a CryptoKey. Import as AES-GCM
        // material (any symmetric algo would work for the wrap; we
        // pick a real one to verify round-trip imports cleanly).
        const innerKey = await crypto.subtle.importKey(
            "raw", keyData, { name: "AES-GCM" }, true, ["encrypt", "decrypt"]);
        const wrapped = await crypto.subtle.wrapKey("raw", innerKey, kek, { name: "AES-KW" });
        results.push(["kw-rfc3394-4-1-wire",
            bytesToHex(wrapped) ===
            "1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5"]);
    }

    // 8. RFC 3394 §4.6: Wrap 256-bit key with 256-bit KEK + unwrap roundtrip.
    {
        const kekBytes = hexToBytes("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        const keyData = hexToBytes("00112233445566778899aabbccddeeff000102030405060708090a0b0c0d0e0f");
        const kek = await crypto.subtle.importKey(
            "raw", kekBytes, { name: "AES-KW" }, false, ["wrapKey", "unwrapKey"]);
        const innerKey = await crypto.subtle.importKey(
            "raw", keyData, { name: "AES-GCM" }, true, ["encrypt", "decrypt"]);
        const wrapped = await crypto.subtle.wrapKey("raw", innerKey, kek, { name: "AES-KW" });
        const expected = "28c9f404c4b810f4cbccb35cfb87f8263f5786e2d80ed326cbc7f0e71a99f43bfb988b9b7a02dd21";
        // Round-trip via unwrap → AES-GCM CryptoKey, then encrypt to compare with reference.
        const unwrapped = await crypto.subtle.unwrapKey(
            "raw", wrapped, kek, { name: "AES-KW" },
            { name: "AES-GCM" }, true, ["encrypt"]);
        const iv = new Uint8Array(12);
        const pt = new TextEncoder().encode("probe");
        // Use the original key bytes via re-import to get a reference ciphertext.
        const refKey = await crypto.subtle.importKey(
            "raw", keyData, { name: "AES-GCM" }, false, ["encrypt"]);
        const refCt = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, refKey, pt);
        const unwrappedCt = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, unwrapped, pt);
        results.push(["kw-rfc3394-4-6-roundtrip",
            bytesToHex(wrapped) === expected &&
            bytesToHex(unwrappedCt) === bytesToHex(refCt)]);
    }

    // 9. AES-KW integrity check rejects tampered wrapped key.
    {
        const kekBytes = hexToBytes("000102030405060708090a0b0c0d0e0f");
        const keyData = hexToBytes("00112233445566778899aabbccddeeff");
        const kek = await crypto.subtle.importKey(
            "raw", kekBytes, { name: "AES-KW" }, false, ["wrapKey", "unwrapKey"]);
        const innerKey = await crypto.subtle.importKey(
            "raw", keyData, { name: "AES-GCM" }, true, ["encrypt"]);
        const wrapped = new Uint8Array(
            await crypto.subtle.wrapKey("raw", innerKey, kek, { name: "AES-KW" }));
        wrapped[0] ^= 0x01;
        let rejected = false;
        try {
            await crypto.subtle.unwrapKey(
                "raw", wrapped, kek, { name: "AES-KW" },
                { name: "AES-GCM" }, true, ["encrypt"]);
        } catch (_) { rejected = true; }
        results.push(["kw-integrity-rejects-tampered", rejected]);
    }

    // 10. Algorithm-name normalization across modes.
    {
        const enc = new TextEncoder();
        const k1 = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "AES-CBC" }, false, ["encrypt"]);
        const k2 = await crypto.subtle.importKey(
            "raw", new Uint8Array(16), { name: "aes-cbc" }, false, ["encrypt"]);
        const iv = new Uint8Array(16);
        const ct1 = await crypto.subtle.encrypt({ name: "AES-CBC", iv }, k1, enc.encode("x"));
        const ct2 = await crypto.subtle.encrypt({ name: "aes-cbc", iv }, k2, enc.encode("x"));
        results.push(["alg-name-normalized", bytesToHex(ct1) === bytesToHex(ct2)]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
