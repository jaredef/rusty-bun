// Tier-J consumer #25: SHA-1 + HMAC-SHA-1 surface (M9.bis).
//
// Validates the E.8 extension to SHA-1. Legacy but still in use:
//   - OAuth 1.0 signature method HMAC-SHA1
//   - AWS Signature Version 2 (still on some legacy S3 endpoints)
//   - Git object identification (SHA-1 of content)
//   - Older Stripe / GitHub / Slack webhook signature schemes
//
// Test vectors from FIPS 180-1 + RFC 2202 — wire-level reproducibility
// is the strongest assertion this fixture makes.

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

    // 1. SHA-1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709 (FIPS 180-1).
    const emptyDigest = await crypto.subtle.digest("SHA-1", enc.encode(""));
    results.push(["sha1-empty",
        bytesToHex(emptyDigest) === "da39a3ee5e6b4b0d3255bfef95601890afd80709"]);

    // 2. SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d.
    const abcDigest = await crypto.subtle.digest("SHA-1", enc.encode("abc"));
    results.push(["sha1-abc",
        bytesToHex(abcDigest) === "a9993e364706816aba3e25717850c26c9cd0d89d"]);

    // 3. SHA-1 output is 20 bytes.
    results.push(["sha1-output-length",
        emptyDigest instanceof ArrayBuffer && emptyDigest.byteLength === 20]);

    // 4. SHA-1 of "The quick brown fox..." canonical short-message vector.
    const foxDigest = await crypto.subtle.digest(
        "SHA-1", enc.encode("The quick brown fox jumps over the lazy dog"));
    results.push(["sha1-fox",
        bytesToHex(foxDigest) === "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"]);

    // 5. Algorithm name normalization: { name: "SHA-1" } object form.
    const objAlg = await crypto.subtle.digest({ name: "SHA-1" }, enc.encode("abc"));
    results.push(["sha1-object-alg",
        bytesToHex(objAlg) === "a9993e364706816aba3e25717850c26c9cd0d89d"]);

    // 6. importKey for HMAC-SHA-1 succeeds with hash:"SHA-1".
    const key = await crypto.subtle.importKey(
        "raw",
        new Uint8Array(20).fill(0x0b),
        { name: "HMAC", hash: "SHA-1" },
        false,
        ["sign", "verify"],
    );
    results.push(["hmac-sha1-importKey",
        key.algorithm.name === "HMAC" &&
        key.algorithm.hash.name === "SHA-1" &&
        key.usages.includes("sign") && key.usages.includes("verify")]);

    // 7. RFC 2202 Test Case 1 wire vector: HMAC-SHA-1 of "Hi There" with
    // key 0x0b*20 = b617318655057264e28bc0b6fb378c8ef146be00.
    const sig = await crypto.subtle.sign("HMAC", key, enc.encode("Hi There"));
    results.push(["rfc2202-test1-vector",
        bytesToHex(sig) === "b617318655057264e28bc0b6fb378c8ef146be00"]);

    // 8. HMAC-SHA-1 output is 20 bytes (SHA-1 width).
    results.push(["hmac-sha1-output-length",
        sig instanceof ArrayBuffer && sig.byteLength === 20]);

    // 9. Verify roundtrip.
    const ok = await crypto.subtle.verify("HMAC", key, sig, enc.encode("Hi There"));
    results.push(["hmac-sha1-verify-ok", ok === true]);

    // 10. Tampered message fails verify.
    const tamperedOk = await crypto.subtle.verify(
        "HMAC", key, sig, enc.encode("Hi there"));  // lowercase t
    results.push(["hmac-sha1-verify-tampered", tamperedOk === false]);

    // 11. RFC 2202 Test Case 2 wire vector: key="Jefe", data="what do ya
    //     want for nothing?", HMAC = effcdf6ae5eb2fa2d27416d5f184df9c259a7c79
    const jefeKey = await crypto.subtle.importKey(
        "raw", enc.encode("Jefe"),
        { name: "HMAC", hash: "SHA-1" }, false, ["sign"]);
    const jefeSig = await crypto.subtle.sign("HMAC", jefeKey,
        enc.encode("what do ya want for nothing?"));
    results.push(["rfc2202-test2-vector",
        bytesToHex(jefeSig) === "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"]);

    // 12. SHA-1 and SHA-256 are distinct surfaces — importing with
    // different hashes pins the key to that hash and they don't interop.
    const sha256Key = await crypto.subtle.importKey(
        "raw", new Uint8Array(20).fill(0x0b),
        { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
    const sha256Sig = await crypto.subtle.sign("HMAC", sha256Key, enc.encode("Hi There"));
    results.push(["hash-isolation",
        sha256Sig.byteLength === 32 &&  // SHA-256 is 32 bytes, distinct from 20
        bytesToHex(sha256Sig) !== bytesToHex(sig)]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
