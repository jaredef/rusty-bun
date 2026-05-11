// Tier-J consumer #29: AES-GCM authenticated encryption.
//
// Validates E.8 extension to AES-GCM. Real consumer use cases: JWE
// A128GCM/A256GCM, encrypted-at-rest cookies, sealed-box envelopes,
// JOSE/COSE encrypted tokens. Test vectors from NIST SP 800-38D
// Appendix B (Test Cases 2, 3, 4).

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

async function importGcm(keyBytes, usages = ["encrypt", "decrypt"]) {
    return crypto.subtle.importKey("raw", keyBytes, { name: "AES-GCM" }, false, usages);
}

async function selfTest() {
    const results = [];

    // ── NIST SP 800-38D Appendix B ────────────────────────────────────

    // Test Case 2: K=0, IV=0, P=16 zero bytes, A=empty.
    // Expected ct||tag = 0388dace... || ab6e47d4...
    {
        const key = await importGcm(hexToBytes("00000000000000000000000000000000"), ["encrypt"]);
        const out = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: hexToBytes("000000000000000000000000") },
            key,
            hexToBytes("00000000000000000000000000000000"),
        );
        results.push(["nist-tc2",
            out instanceof ArrayBuffer && out.byteLength === 32 &&
            bytesToHex(out) ===
            "0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf"]);
    }

    // Test Case 3: standard JOSE-shaped vector with no AAD.
    {
        const key = await importGcm(hexToBytes("feffe9928665731c6d6a8f9467308308"), ["encrypt"]);
        const pt = hexToBytes(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72" +
            "1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255");
        const out = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: hexToBytes("cafebabefacedbaddecaf888") },
            key, pt,
        );
        results.push(["nist-tc3",
            bytesToHex(out) ===
            "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e" +
            "21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f5985" +
            "4d5c2af327cd64a62cf35abd2ba6fab4"]);
    }

    // Test Case 4: with additionalData (AAD).
    {
        const key = await importGcm(hexToBytes("feffe9928665731c6d6a8f9467308308"), ["encrypt"]);
        const pt = hexToBytes(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72" +
            "1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39");
        const aad = hexToBytes("feedfacedeadbeeffeedfacedeadbeefabaddad2");
        const out = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: hexToBytes("cafebabefacedbaddecaf888"),
              additionalData: aad },
            key, pt,
        );
        results.push(["nist-tc4-with-aad",
            bytesToHex(out) ===
            "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e" +
            "21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091" +
            "5bc94fbc3221a5db94fae95ae7121a47"]);
    }

    // ── Round-trip behavior ───────────────────────────────────────────

    // 4. Encrypt → decrypt round-trip preserves plaintext.
    {
        const enc = new TextEncoder();
        const dec = new TextDecoder();
        const keyBytes = new Uint8Array(16);
        for (let i = 0; i < 16; i++) keyBytes[i] = i;
        const key = await importGcm(keyBytes);
        const iv = new Uint8Array(12);
        for (let i = 0; i < 12; i++) iv[i] = i + 100;
        const plaintext = "hello AES-GCM, this is a round-trip test";
        const ct = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv }, key, enc.encode(plaintext));
        const pt = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv }, key, ct);
        results.push(["roundtrip", dec.decode(pt) === plaintext]);
    }

    // 5. AAD-bound round-trip: decrypt with matching AAD succeeds.
    {
        const enc = new TextEncoder();
        const dec = new TextDecoder();
        const key = await importGcm(new Uint8Array(32));  // AES-256
        const iv = new Uint8Array(12);
        const aad = enc.encode("context-string-bound-to-ciphertext");
        const ct = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv, additionalData: aad }, key, enc.encode("aad-roundtrip"));
        const pt = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv, additionalData: aad }, key, ct);
        results.push(["aad-roundtrip", dec.decode(pt) === "aad-roundtrip"]);
    }

    // 6. Tampered ciphertext is rejected.
    {
        const key = await importGcm(new Uint8Array(16));
        const iv = new Uint8Array(12);
        const ct = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv }, key, new TextEncoder().encode("secret"));
        const tampered = new Uint8Array(ct);
        tampered[0] ^= 0x01;
        let rejected = false;
        try {
            await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, tampered);
        } catch (_) { rejected = true; }
        results.push(["tampered-rejected", rejected]);
    }

    // 7. Tag mutation is rejected.
    {
        const key = await importGcm(new Uint8Array(16));
        const iv = new Uint8Array(12);
        const ct = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv }, key, new TextEncoder().encode("secret"));
        const tampered = new Uint8Array(ct);
        tampered[tampered.length - 1] ^= 0x01;
        let rejected = false;
        try {
            await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, tampered);
        } catch (_) { rejected = true; }
        results.push(["tag-tampered-rejected", rejected]);
    }

    // 8. Wrong AAD is rejected.
    {
        const enc = new TextEncoder();
        const key = await importGcm(new Uint8Array(16));
        const iv = new Uint8Array(12);
        const ct = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv, additionalData: enc.encode("right-aad") },
            key, enc.encode("payload"));
        let rejected = false;
        try {
            await crypto.subtle.decrypt(
                { name: "AES-GCM", iv, additionalData: enc.encode("wrong-aad") },
                key, ct);
        } catch (_) { rejected = true; }
        results.push(["wrong-aad-rejected", rejected]);
    }

    // 9. AES-128 vs AES-256 keys are isolated.
    {
        const enc = new TextEncoder();
        const k128 = await importGcm(new Uint8Array(16));
        const k256 = await importGcm(new Uint8Array(32));
        const iv = new Uint8Array(12);
        const ct1 = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, k128, enc.encode("same"));
        const ct2 = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, k256, enc.encode("same"));
        results.push(["key-size-isolation",
            ct1.byteLength === ct2.byteLength &&
            bytesToHex(ct1) !== bytesToHex(ct2)]);
    }

    // 10. Different IVs produce different ciphertext.
    {
        const enc = new TextEncoder();
        const key = await importGcm(new Uint8Array(16));
        const iv1 = new Uint8Array(12); iv1[0] = 1;
        const iv2 = new Uint8Array(12); iv2[0] = 2;
        const ct1 = await crypto.subtle.encrypt({ name: "AES-GCM", iv: iv1 }, key, enc.encode("same"));
        const ct2 = await crypto.subtle.encrypt({ name: "AES-GCM", iv: iv2 }, key, enc.encode("same"));
        results.push(["iv-sensitive", bytesToHex(ct1) !== bytesToHex(ct2)]);
    }

    // 11. Algorithm normalization: lower-case + dashes both accepted.
    {
        const enc = new TextEncoder();
        const key = await importGcm(new Uint8Array(16));
        const iv = new Uint8Array(12);
        const ct1 = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, enc.encode("x"));
        const ct2 = await crypto.subtle.encrypt({ name: "aes-gcm", iv }, key, enc.encode("x"));
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
