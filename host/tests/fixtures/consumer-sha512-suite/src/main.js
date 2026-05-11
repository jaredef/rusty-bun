// Tier-J consumer #26: SHA-384 + SHA-512 surface (M9.bis).
//
// Validates the E.8 extension to SHA-384 + SHA-512. JWT HS384/HS512,
// higher-tier OAuth, FIPS 140-3 contexts. Test vectors from FIPS 180-4
// (digest) + RFC 4231 (HMAC) — wire-level reproducibility.

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

    // ── SHA-512 digest vectors ────────────────────────────────────────

    // 1. SHA-512("") = cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
    const empty512 = await crypto.subtle.digest("SHA-512", enc.encode(""));
    results.push(["sha512-empty",
        empty512 instanceof ArrayBuffer && empty512.byteLength === 64 &&
        bytesToHex(empty512) === "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"]);

    // 2. SHA-512("abc")
    const abc512 = await crypto.subtle.digest("SHA-512", enc.encode("abc"));
    results.push(["sha512-abc",
        bytesToHex(abc512) === "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"]);

    // ── SHA-384 digest vectors ────────────────────────────────────────

    // 3. SHA-384("") = 38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b
    const empty384 = await crypto.subtle.digest("SHA-384", enc.encode(""));
    results.push(["sha384-empty",
        empty384.byteLength === 48 &&
        bytesToHex(empty384) === "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"]);

    // 4. SHA-384("abc")
    const abc384 = await crypto.subtle.digest("SHA-384", enc.encode("abc"));
    results.push(["sha384-abc",
        bytesToHex(abc384) === "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"]);

    // ── HMAC-SHA-512 (RFC 4231 vectors) ───────────────────────────────

    // 5. RFC 4231 Test 1: key=0x0b*20, data="Hi There"
    const key512 = await crypto.subtle.importKey(
        "raw", new Uint8Array(20).fill(0x0b),
        { name: "HMAC", hash: "SHA-512" }, false, ["sign", "verify"]);
    const sig512 = await crypto.subtle.sign("HMAC", key512, enc.encode("Hi There"));
    results.push(["hmac-sha512-rfc4231-test1",
        sig512.byteLength === 64 &&
        bytesToHex(sig512) === "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854"]);

    // 6. RFC 4231 Test 2: key="Jefe", data="what do ya want for nothing?"
    const jefe512 = await crypto.subtle.importKey(
        "raw", enc.encode("Jefe"),
        { name: "HMAC", hash: "SHA-512" }, false, ["sign"]);
    const jefeSig512 = await crypto.subtle.sign("HMAC", jefe512,
        enc.encode("what do ya want for nothing?"));
    results.push(["hmac-sha512-rfc4231-test2",
        bytesToHex(jefeSig512) === "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"]);

    // ── HMAC-SHA-384 (RFC 4231 vectors) ───────────────────────────────

    // 7. RFC 4231 Test 1 for HMAC-SHA-384
    const key384 = await crypto.subtle.importKey(
        "raw", new Uint8Array(20).fill(0x0b),
        { name: "HMAC", hash: "SHA-384" }, false, ["sign", "verify"]);
    const sig384 = await crypto.subtle.sign("HMAC", key384, enc.encode("Hi There"));
    results.push(["hmac-sha384-rfc4231-test1",
        sig384.byteLength === 48 &&
        bytesToHex(sig384) === "afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59cfaea9ea9076ede7f4af152e8b2fa9cb6"]);

    // 8. Verify roundtrip SHA-512.
    const ok512 = await crypto.subtle.verify("HMAC", key512, sig512, enc.encode("Hi There"));
    results.push(["hmac-sha512-verify-ok", ok512 === true]);

    // 9. Verify roundtrip SHA-384.
    const ok384 = await crypto.subtle.verify("HMAC", key384, sig384, enc.encode("Hi There"));
    results.push(["hmac-sha384-verify-ok", ok384 === true]);

    // 10. Tampered message fails verify for SHA-512.
    const tamperedOk512 = await crypto.subtle.verify("HMAC", key512, sig512,
        enc.encode("Hi there"));  // lowercase t
    results.push(["hmac-sha512-tamper-rejected", tamperedOk512 === false]);

    // 11. Hash-isolation: SHA-512 key and SHA-384 key from the same key
    // bytes produce distinct outputs and distinct lengths.
    results.push(["hash-isolation-384-vs-512",
        sig512.byteLength === 64 && sig384.byteLength === 48 &&
        bytesToHex(sig512) !== bytesToHex(sig384)]);

    // 12. Cross-hash signatures don't cross-verify. A SHA-512 signature
    // verified against a SHA-384 key (or vice versa) must fail. We can't
    // import the wrong-shaped signature directly, so check via length and
    // bytes mismatch — verify returns false.
    // Use a fresh data run to avoid any cached state.
    const sig384AgainstData = await crypto.subtle.sign("HMAC", key384, enc.encode("test"));
    const sig512AgainstData = await crypto.subtle.sign("HMAC", key512, enc.encode("test"));
    // verify uses the key's hash; we pass the wrong-hash signature.
    const crossA = await crypto.subtle.verify("HMAC", key384, sig512AgainstData, enc.encode("test"));
    const crossB = await crypto.subtle.verify("HMAC", key512, sig384AgainstData, enc.encode("test"));
    results.push(["cross-hash-rejected",
        crossA === false && crossB === false]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
