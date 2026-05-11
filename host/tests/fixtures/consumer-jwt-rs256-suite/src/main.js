// Tier-J consumer #37: JWS JWT RS256 (RSASSA-PKCS1-v1_5 + SHA-256).
//
// The canonical asymmetric JWT signature. Used by Auth0, Okta, AWS
// Cognito, Google OAuth, Apple Sign-In, every major IdP. RS256 is
// deterministic (no nonce), so cross-engine byte-equal signatures
// are the strong differential signal.
//
// This fixture is a real JWS-shaped JWT verifier: it constructs a
// JWT the way a production token issuer does, signs it, parses the
// way a relying party does, and verifies the signature.

const JWK_PRIVATE = {
    kty: "RSA",
    n: "oCaBzo9bpvtYrQxCikUkuNwLFipKwV8Fbxq2N7BCGToOPiRbA8g4KX_F2CfvDgySJ9iYDmepHGGdFG4BPIR4syLiQQdB5CWXopr0ixSewTJATWB29TgDgGDNAFmxgmmX8LY5b9kao3a8WGz9cOJ10OhVgJse-QcDjwhvHbkCBJ5fM6Y2aBAXcb06RqZfin9S5cVAzxee7Bealk1VnB3sn05aasB2UIu6v2KmmKG0cxoF0Uqo3Ob_vvVlxOIgBpwlH0m0_WrRm2K-F77HWrOsxZLrlbAvGe59rx6PF7H5o3-gyCdUSIUq5tTfSGlcgakhFBBlUnyc7-CQLChBq2jp7Q",
    e: "AQAB",
    d: "G6rLXh4SHWTqyuqFTFHpqC4LlEa2J3X9AFbDCBfhM25-K2oodxzN5w115oPvnqO5VDzs-AAcjRSoCHGAsS3JlFkAcW-JiJqd-a9_c8-aJZJC3Zs-sdp9cF5IzDiym-8WGrXAcnw1R-wpWbVqi2f0JqUcsF0cGrWFfd5dI-tkV9umZWz5rS-W4MU-N674oHkCgAhxgqyt15bvLy7pxwWSu9Vg4fvyZRENO3jY6tC01BFZI_YAiTv_aVp6FjlbXZ5OIarMMmYj1rJs3CH3BPyBp4GNo6KTvRediVE68Yohj_XP6R9dfY83LRttaxmE9suMKGR4YXzxq2WDieznyNoXwQ",
    p: "0FjFjUQ45Ba6J5PlssOKTeGn-Fg1LhZdRf6HsmpkhMNl32xrTREn15I-bzfoFDHAy0k4iEvDnbLOxQSw1XPvN1JHyIYhfxmdilamGDfMSICbd0nICHPI5FI48CbFQq1AkD7FPIW_B6BQh-kfgpdnM-_3on1MYdJbD14RxjzBRXk",
    q: "xMe3wWAMG8lwzcJq7XTu5IX4Qb8N14b6tWjrjoeGQ_dmf0I8pHQtKZMdVXU4EuuDh2D5MuYkluGN-N86Kf3EleP1iTj2mz_r7AUi7GPl7hrWgcp6Oy_oIOqaulP7KSUFDE9Vu8JOSJ8MIVIEbzrgUP3qWJui7qNCsIbO8q_1LxU",
    dp: "t6BV1RD410aUoUc_nlOrNMMayM3taQY7BPK1ZHFS0JRq2AT1eUISjHOPZXSvrgS_uCt7kNy9tuKeTJS6yhZZErgJHlnhceUAral60EN1X71ByFwV2iU6PMme90Ikf4S1L6yzJ4l1eWI5N-AmbnHEeskXx3WJeXnt2dh97-siKHk",
    dq: "ML5wLDyIg1GQ2ccxCYUPsBfneRHEcgEDlXBZ-UJk5e0gvKFBuFL25PeGKqcQrs8cE6rXz93mbmGM83sIQ2KTEbYYGle77pUU8bAMCJZuXF7Vh-0J_iNN0umKTmGDM5vx9iyoxgvJrH-JV6-jXZIuAqIDLNPQtmBGoseh98fSakk",
    qi: "G-5SzpRhoOz0XiqdBBEhe3l8vyoalmQAK4cKzFad4-6qjynOzskFEjCmgExH37BUc_6NCee0z63tSZEJK5OCCMvPKXbdrboqBe1XsPHiARz3VZKeFKUXqi9M62H9V9acNsMcitaovW8r9Pfo0dSQ3jXvMTqwXA26m5yU8TZFEcM",
};
const JWK_PUBLIC = { kty: "RSA", n: JWK_PRIVATE.n, e: JWK_PRIVATE.e };

// Base64url encode/decode for JWS Compact Serialization.
function b64uEncodeBytes(bytes) {
    const u8 = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes;
    let bin = "";
    for (let i = 0; i < u8.length; i++) bin += String.fromCharCode(u8[i]);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
function b64uDecode(s) {
    const b64 = s.replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - s.length % 4) % 4);
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
}
function b64uEncodeStr(s) {
    return b64uEncodeBytes(new TextEncoder().encode(s));
}

async function selfTest() {
    const results = [];
    const enc = new TextEncoder();

    const privKey = await crypto.subtle.importKey(
        "jwk", JWK_PRIVATE,
        { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
        false, ["sign"]);
    const pubKey = await crypto.subtle.importKey(
        "jwk", JWK_PUBLIC,
        { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
        false, ["verify"]);
    results.push(["import-jwk",
        privKey.type === "private" && pubKey.type === "public" &&
        privKey.algorithm.name === "RSASSA-PKCS1-v1_5"]);

    // 1. Standard JWT round-trip: header.payload.signature
    const header = { alg: "RS256", typ: "JWT" };
    const payload = { sub: "user-42", iss: "rusty-bun-test", iat: 1700000000, exp: 1900000000 };
    const headerB64 = b64uEncodeStr(JSON.stringify(header));
    const payloadB64 = b64uEncodeStr(JSON.stringify(payload));
    const signingInput = headerB64 + "." + payloadB64;
    const sig = await crypto.subtle.sign(
        { name: "RSASSA-PKCS1-v1_5" }, privKey, enc.encode(signingInput));
    const sigB64 = b64uEncodeBytes(sig);
    const jwt = signingInput + "." + sigB64;
    results.push(["jwt-shape-3-segments", jwt.split(".").length === 3]);
    results.push(["jwt-signature-256-bytes", sig.byteLength === 256]);

    // 2. Verify the just-built JWT.
    const [hSeg, pSeg, sSeg] = jwt.split(".");
    const ok = await crypto.subtle.verify(
        { name: "RSASSA-PKCS1-v1_5" }, pubKey,
        b64uDecode(sSeg), enc.encode(hSeg + "." + pSeg));
    results.push(["jwt-verify-ok", ok === true]);

    // 3. Tampered payload rejection.
    const tamperedPayload = b64uEncodeStr(JSON.stringify(
        { ...payload, sub: "user-attacker" }));
    const tamperedInput = headerB64 + "." + tamperedPayload;
    const tamperedOk = await crypto.subtle.verify(
        { name: "RSASSA-PKCS1-v1_5" }, pubKey,
        b64uDecode(sSeg), enc.encode(tamperedInput));
    results.push(["tampered-payload-rejected", tamperedOk === false]);

    // 4. Tampered signature rejection.
    const sigBytes = new Uint8Array(b64uDecode(sSeg));
    sigBytes[0] ^= 0x01;
    const sigTampOk = await crypto.subtle.verify(
        { name: "RSASSA-PKCS1-v1_5" }, pubKey, sigBytes, enc.encode(signingInput));
    results.push(["tampered-signature-rejected", sigTampOk === false]);

    // 5. Determinism: two signs of same input produce identical signatures
    //    (RS256 has no nonce — this is the distinguishing trait from PSS).
    const sig2 = await crypto.subtle.sign(
        { name: "RSASSA-PKCS1-v1_5" }, privKey, enc.encode(signingInput));
    const a = new Uint8Array(sig);
    const b = new Uint8Array(sig2);
    let equal = a.length === b.length;
    for (let i = 0; equal && i < a.length; i++) if (a[i] !== b[i]) equal = false;
    results.push(["deterministic", equal]);

    // 6. Wrong-hash mismatch: importing the key for SHA-384 then verifying
    //    a SHA-256-signed JWT must fail. Tests that the hash binding flows
    //    through importKey.
    const pubKeySha384 = await crypto.subtle.importKey(
        "jwk", JWK_PUBLIC,
        { name: "RSASSA-PKCS1-v1_5", hash: "SHA-384" },
        false, ["verify"]);
    const wrongHashOk = await crypto.subtle.verify(
        { name: "RSASSA-PKCS1-v1_5" }, pubKeySha384,
        b64uDecode(sSeg), enc.encode(signingInput));
    results.push(["hash-binding", wrongHashOk === false]);

    // 7. Multi-hash family: RS384 and RS512 round-trips.
    for (const [alg, hashSpec] of [["RS384", "SHA-384"], ["RS512", "SHA-512"]]) {
        const p = await crypto.subtle.importKey(
            "jwk", JWK_PRIVATE, { name: "RSASSA-PKCS1-v1_5", hash: hashSpec }, false, ["sign"]);
        const pu = await crypto.subtle.importKey(
            "jwk", JWK_PUBLIC, { name: "RSASSA-PKCS1-v1_5", hash: hashSpec }, false, ["verify"]);
        const s = await crypto.subtle.sign(
            { name: "RSASSA-PKCS1-v1_5" }, p, enc.encode(signingInput));
        const v = await crypto.subtle.verify(
            { name: "RSASSA-PKCS1-v1_5" }, pu, s, enc.encode(signingInput));
        results.push([alg.toLowerCase() + "-roundtrip", v === true && s.byteLength === 256]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
