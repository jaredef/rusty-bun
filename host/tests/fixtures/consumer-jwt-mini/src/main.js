// Tier-J consumer #24: vendored JWT HS256 library exercising the just-
// closed E.8 WebCrypto surface from inside a third-party-library context.
//
// jwt-mini internally calls crypto.subtle.importKey, .sign, .verify.
// This fixture verifies the surface works when invoked from npm-shape
// code (not engagement-author code) — strong evidence that real JWT
// libraries (jsonwebtoken, jose, fast-jwt) would run identically.

import { sign, verify } from "jwt-mini";
import jwt from "jwt-mini";

const SECRET = "my-very-secret-key";

async function selfTest() {
    const results = [];

    // 1. Round-trip: sign a payload, verify produces same payload.
    const payload = { sub: "user-42", iat: 0, scope: ["read"] };
    const token = await sign(payload, SECRET);
    const result = await verify(token, SECRET);
    results.push(["round-trip",
        result.valid === true &&
        result.payload.sub === "user-42" &&
        result.payload.iat === 0 &&
        result.payload.scope.length === 1 &&
        result.payload.scope[0] === "read"]);

    // 2. Token shape: three dot-separated base64url segments.
    const parts = token.split(".");
    const isBase64Url = (s) => /^[A-Za-z0-9_-]+$/.test(s);
    results.push(["token-shape",
        parts.length === 3 && parts.every(isBase64Url)]);

    // 3. Header is decodable and announces HS256.
    function b64urlToBytes(s) {
        s = s.replace(/-/g, "+").replace(/_/g, "/");
        while (s.length % 4) s += "=";
        const bin = atob(s);
        const out = new Uint8Array(bin.length);
        for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
        return out;
    }
    const dec = new TextDecoder();
    const header = JSON.parse(dec.decode(b64urlToBytes(parts[0])));
    results.push(["header-alg", header.alg === "HS256" && header.typ === "JWT"]);

    // 4. Wrong secret fails verification.
    const wrong = await verify(token, "different-secret");
    results.push(["wrong-secret",
        wrong.valid === false && wrong.error === "signature mismatch"]);

    // 5. Tampered payload (flip a byte in the payload segment) fails verify.
    const tamperedToken = parts[0] + "." + parts[1].slice(0, -1) +
        (parts[1].slice(-1) === "A" ? "B" : "A") + "." + parts[2];
    const tampered = await verify(tamperedToken, SECRET);
    results.push(["tampered-payload", tampered.valid === false]);

    // 6. Tampered signature fails verify.
    const tamperedSig = parts[0] + "." + parts[1] + "." +
        (parts[2].slice(0, -1) + (parts[2].slice(-1) === "A" ? "B" : "A"));
    const tsig = await verify(tamperedSig, SECRET);
    results.push(["tampered-signature", tsig.valid === false]);

    // 7. Malformed token (not three parts) rejected.
    const malformed = await verify("abc.def", SECRET);
    results.push(["malformed-token",
        malformed.valid === false && /malformed/.test(malformed.error)]);

    // 8. Determinism: same payload + secret → same token.
    const token2 = await sign(payload, SECRET);
    results.push(["sign-deterministic", token === token2]);

    // 9. Different payloads → different tokens.
    const otherToken = await sign({ sub: "user-99", iat: 0, scope: ["read"] }, SECRET);
    results.push(["different-payload-different-token", token !== otherToken]);

    // 10. Default export gives the same functions.
    const defaultRoundTrip = await jwt.verify(
        await jwt.sign({ ok: true }, SECRET), SECRET);
    results.push(["default-export",
        defaultRoundTrip.valid === true && defaultRoundTrip.payload.ok === true]);

    // 11. Cross-library reproducibility: build a known token by hand and
    // verify it. Uses base64url('{"alg":"HS256","typ":"JWT"}') and a
    // payload string; signature computed via crypto.subtle directly.
    // This is the "every JWT library in the world produces the same HS256
    // output for the same input" check.
    const knownHeader = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";  // base64url
    const knownPayload = "eyJzdWIiOiJBbGljZSJ9";  // base64url of {"sub":"Alice"}
    const enc = new TextEncoder();
    const signingInput = knownHeader + "." + knownPayload;
    const key = await crypto.subtle.importKey(
        "raw", enc.encode(SECRET),
        { name: "HMAC", hash: "SHA-256" }, false, ["sign", "verify"]);
    const sig = new Uint8Array(await crypto.subtle.sign("HMAC", key, enc.encode(signingInput)));
    let sigB64 = "";
    for (let i = 0; i < sig.length; i++) sigB64 += String.fromCharCode(sig[i]);
    sigB64 = btoa(sigB64).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
    const handcraftedToken = signingInput + "." + sigB64;
    const handcraftedVerify = await verify(handcraftedToken, SECRET);
    results.push(["hand-crafted-interop",
        handcraftedVerify.valid === true &&
        handcraftedVerify.payload.sub === "Alice"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
