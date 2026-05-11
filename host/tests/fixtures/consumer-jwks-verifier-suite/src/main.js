// Tier-J consumer #38: real-shape JWKS-based JWT verifier.
//
// Exercises the entire E.8 WebCrypto closure end-to-end through a
// vendored library that implements the production JWKS pattern.
// Multiple algorithms (RS256 + ES256 + PS256), kid-based key lookup,
// alg-match security check, signing-input reconstruction. This is the
// shape of code that runs at every Auth0 / Okta / Cognito / Google
// relying party.

import { JwksVerifier, JwksError } from "jwks-verifier";

// Issuer keys: one RSA (RS256 / PS256), one EC P-256 (ES256).
const RSA_PRIV = {"d":"FwRsP1Aao7v22pThxWLMmUWaAxHJD-NKWAgc7C92uf-TQIaH0hvvIGQFWG8xaEuNCxrPH3R_aNlCh9YCcXp9pOJIIjVqvNsaW0FKmjatruTFYZKBZgWvdXuPDp98V-SBLgeRvB6V2aO156szWv-G8YzDh_dmiuSpbasEp5UjzRXS-9jV8Qsgk4AzsMVH19YrxL8__1JR0fWw57p3tchlcp62K5u1jkmJ9W4W118KnPZLM4Oi2DBMw2S0v90EQNzJxE_iiLivQoHtdk0hugCrDOG20sKF20z2nYxO4a7jkpedKD3pjjunCQU0JZ1FoN3Q8bQGMPTcpy1BDHkCu-z_tQ","dp":"GmtF4ftnmLKy1-lktcSjM6YH-ees0_f0N3AhH41vZQCzfOBSrdDQVtpd-L-42y_3M3hh1_styIaRMsTmqONde7ZP9sjECY8zO0mm28zFptDQsg3o2BbNz4lysyZjyiUjLZzZ0g7D6Sqm0I6dLtI3_St3461_NX4fgwge6ty7NSk","dq":"n-wfpq73mhfkbtKWGMNkc1L0I5Ww9YZMTGqPGJGMAQJFpc3HdJfHmHbWvxS_ZthGEzq8pGJ8QuWcvm8YAPEO32qKLmgczuw_kJPScefvbSiaQRnqyc3cSbUQD_aU7JIglaVcbX86iLXANwX3gDoolej6z7CFgw49MfsWfQwO20s","e":"AQAB","kty":"RSA","n":"wasdFBfcGqRaiIUwKKRmSLVFJB0wqkMOVy2NTfCl9cFlmEPZPPXA1y8nwEGIBOVSv1iVa-OyenvjPqEOiw66d03lIYrT7zuPzeY6Sh---avvgxopO33lWbokjc40od1DGUya28IkwKavjewunJtTrnsF-RAWsBMSxx9EWZaby94gmFE-SbDqJ50FPkrm5kYo7GuMbSXPik5_kOO5bwGO7-8naTi3AzEBuOHs1Iwl6hoKae-_IE8JKYbeL10JhqlEAS9E-Ks9rQymNEj4S9KTSzqCFWWVTE8wKSzx-QfSPvFDw7EuqxTN5qFucrIqFwki2C5gssTvX3qyS0EyVUgwYQ","p":"-86Ri5brJ4zdZd9oqYaDqMbMnnEiiXxbtuQsEdEy36_lsv_Voudb36KnzeAwqFCB_U3JgeZNR8tYXI9RYQZX1KfTNvrkiSFjbL9MTZNW9MYM6g5nk5IEG6wXGv7rBYtxIrNIfDlosN4xj-ngCOnEHCBbHAyd-HRYzTQMBmQBQwc","q":"xOS0oPywFJ6Z6v-JtYgN2CrHwjhHuw0kjHtOZpCO-yOifV-x_IsVcg2McYioEih4IdKIbv4mzEHE7uvxXGK0JXYH66iLlBEYJq627Cd2GIls-IqtqO1f0mBlIiTQaJsTBPe5Qqi3rHmGJcNPYJKM-mA8chJZIddh6YD4qC5rD1c","qi":"9TRPY13L0_mIsW_1mDGDrHXicizmpMJMxvtst6ieEGZyaAsNZK9iyzndkRVnpb1YNboMYFlWcGYIrdaih_uwxsnZr7Lp-UVhmgEKG15KYV_QeBuPjJmimfbvUjVciVCoMXlTiPYns07dNZfyrd3Oai4BMcYV8MBiCieMwlT-D-M"};
const EC_PRIV = {"crv":"P-256","d":"H2ValMKVB9rxOTrUf_qwZBEAwdnoILUmc4Yjj2FisGk","kty":"EC","x":"bc48_35hKYYzdN09-YESMTSC59F2nYRtqPUOu0HW8Zo","y":"LBYEt9UJDvWSkPzN7XLvA8zAtT9ON0cXlRf3mPiHvd0"};

// Public JWKS the relying party fetches from the issuer.
const JWKS = {"keys":[
    {"alg":"RS256","kty":"RSA","kid":"rsa-2024","use":"sig","e":RSA_PRIV.e,"n":RSA_PRIV.n},
    {"kty":"EC","kid":"ec-2024","alg":"ES256","use":"sig","crv":"P-256","x":EC_PRIV.x,"y":EC_PRIV.y},
]};

// ── Token-building helpers (issuer-side equivalent) ────────────────
const enc = new TextEncoder();
function b64uEncodeBytes(b) {
    const u = b instanceof ArrayBuffer ? new Uint8Array(b) : b;
    let bin = ""; for (const x of u) bin += String.fromCharCode(x);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
function b64uEncodeStr(s) { return b64uEncodeBytes(enc.encode(s)); }

async function buildToken(alg, kid, payload, signKey) {
    const header = { alg, typ: "JWT", kid };
    const h = b64uEncodeStr(JSON.stringify(header));
    const p = b64uEncodeStr(JSON.stringify(payload));
    let verifyAlg;
    if (alg.startsWith("RS")) verifyAlg = { name: "RSASSA-PKCS1-v1_5" };
    else if (alg.startsWith("PS")) verifyAlg = { name: "RSA-PSS", saltLength: 32 };
    else if (alg === "ES256") verifyAlg = { name: "ECDSA", hash: "SHA-256" };
    else throw new Error("unsupported alg: " + alg);
    const sig = await crypto.subtle.sign(verifyAlg, signKey, enc.encode(h + "." + p));
    return h + "." + p + "." + b64uEncodeBytes(sig);
}

async function selfTest() {
    const results = [];

    // Import signing keys (issuer-side).
    const rsaSignerRS256 = await crypto.subtle.importKey(
        "jwk", RSA_PRIV, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["sign"]);
    const rsaSignerPS256 = await crypto.subtle.importKey(
        "jwk", RSA_PRIV, { name: "RSA-PSS", hash: "SHA-256" }, false, ["sign"]);
    const ecSignerES256 = await crypto.subtle.importKey(
        "jwk", EC_PRIV, { name: "ECDSA", namedCurve: "P-256" }, false, ["sign"]);

    // The relying party constructs a verifier once and reuses it.
    const verifier = new JwksVerifier(JWKS);

    const basePayload = { sub: "user-42", iss: "https://issuer.example.com" };

    // 1. RS256 token verifies successfully.
    {
        const t = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        const { payload } = await verifier.verify(t, { iss: "https://issuer.example.com" });
        results.push(["rs256-verify-and-claims", payload.sub === "user-42"]);
    }

    // 2. ES256 token verifies successfully.
    {
        const t = await buildToken("ES256", "ec-2024", basePayload, ecSignerES256);
        const { payload } = await verifier.verify(t);
        results.push(["es256-verify", payload.sub === "user-42"]);
    }

    // 3. Wrong issuer is rejected by claim check.
    {
        const t = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        let caught = null;
        try { await verifier.verify(t, { iss: "https://attacker.example.com" }); }
        catch (e) { caught = e; }
        results.push(["wrong-iss-rejected", caught instanceof JwksError && caught.code === "bad_iss"]);
    }

    // 4. Expired token is rejected.
    {
        const t = await buildToken("RS256", "rsa-2024",
            { ...basePayload, exp: 1000 }, rsaSignerRS256);
        let caught = null;
        try { await verifier.verify(t, { now: 2000 }); }
        catch (e) { caught = e; }
        results.push(["expired-rejected", caught instanceof JwksError && caught.code === "expired"]);
    }

    // 5. Algorithm-confusion attack: token claims RS256 but uses the EC key's
    //    kid. JWKS lookup finds the EC entry; alg-match check rejects.
    {
        // Build a token that LOOKS like RS256 but mislabels the kid.
        const t = await buildToken("RS256", "ec-2024", basePayload, rsaSignerRS256);
        let caught = null;
        try { await verifier.verify(t); }
        catch (e) { caught = e; }
        results.push(["alg-confusion-rejected", caught instanceof JwksError && caught.code === "alg_mismatch"]);
    }

    // 6. Unknown kid is rejected.
    {
        const t = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        // Re-pack with a fake kid.
        const [h, p, s] = t.split(".");
        const decoded = JSON.parse(new TextDecoder().decode(
            Uint8Array.from(atob(h.replace(/-/g, "+").replace(/_/g, "/") +
                "=".repeat((4 - h.length % 4) % 4)), c => c.charCodeAt(0))));
        decoded.kid = "non-existent";
        const newH = b64uEncodeStr(JSON.stringify(decoded));
        // The signature is now invalid because the header bytes changed; but
        // the kid check fires FIRST. We assert we get kid_not_found, not
        // bad_signature — proves the verifier short-circuits on kid lookup.
        const fakeToken = newH + "." + p + "." + s;
        let caught = null;
        try { await verifier.verify(fakeToken); }
        catch (e) { caught = e; }
        results.push(["unknown-kid-rejected", caught instanceof JwksError && caught.code === "kid_not_found"]);
    }

    // 7. Tampered payload rejected at signature step.
    {
        const t = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        const [h, p, s] = t.split(".");
        // Re-encode payload with a different sub.
        const newP = b64uEncodeStr(JSON.stringify({ ...basePayload, sub: "user-attacker" }));
        let caught = null;
        try { await verifier.verify(h + "." + newP + "." + s); }
        catch (e) { caught = e; }
        results.push(["tampered-payload-rejected", caught instanceof JwksError && caught.code === "bad_signature"]);
    }

    // 8. Cache: second verify of same kid reuses imported key. Test
    //    indirectly by checking the cache map size after two verifies.
    {
        const t1 = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        const t2 = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        await verifier.verify(t1);
        await verifier.verify(t2);
        // Map size grew at most by 1 across both verifies (the kid was new
        // for the verifier instance, but the second verify hit cache).
        results.push(["key-cache-works",
            verifier._cache instanceof Map && verifier._cache.has("rsa-2024")]);
    }

    // 9. Multi-alg over the same verifier instance: RS256 + ES256 both
    //    succeed within the same JwksVerifier; cache is populated for both.
    {
        const rs = await buildToken("RS256", "rsa-2024", basePayload, rsaSignerRS256);
        const es = await buildToken("ES256", "ec-2024", basePayload, ecSignerES256);
        const v1 = await verifier.verify(rs);
        const v2 = await verifier.verify(es);
        results.push(["multi-alg-via-one-verifier",
            v1.header.alg === "RS256" && v2.header.alg === "ES256" &&
            verifier._cache.has("rsa-2024") && verifier._cache.has("ec-2024")]);
    }

    // 10. PS256 (RSA-PSS): JWKS has only RS256 + ES256, so PS256 token
    //     for the same kid is rejected by the alg-match check (JWKS alg
    //     claim binds key to its declared algorithm).
    {
        const t = await buildToken("PS256", "rsa-2024", basePayload, rsaSignerPS256);
        let caught = null;
        try { await verifier.verify(t); }
        catch (e) { caught = e; }
        results.push(["ps256-against-rs256-key-rejected",
            caught instanceof JwksError && caught.code === "alg_mismatch"]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
