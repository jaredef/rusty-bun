// Tier-J consumer #32: RSA-OAEP encrypt/decrypt.
//
// Validates E.8 extension to asymmetric crypto. Real consumer use:
// JWE RSA-OAEP-256 envelopes, OAuth2 PoP, encrypted cookies,
// hybrid-encryption schemes (RSA wraps a content key).
//
// OAEP encrypt is RANDOMIZED — Bun and rusty-bun-host produce
// different ciphertexts each run. Verification is mutual round-trip:
//   - encrypt → decrypt (same engine) recovers the plaintext
//   - encrypt-and-immediately-decrypt is the only stable signal under
//     the engine, since cross-engine compare-ciphertext is impossible.
// We also assert structural properties (ciphertext is k bytes; label
// sensitivity; modulus-length agreement).

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

async function selfTest() {
    const results = [];
    const enc = new TextEncoder();
    const dec = new TextDecoder();

    // ── Import: public + private keys via JWK ─────────────────────────
    let pubKey, privKey;
    try {
        pubKey = await crypto.subtle.importKey(
            "jwk", JWK_PUBLIC,
            { name: "RSA-OAEP", hash: "SHA-256" },
            false, ["encrypt"]);
        privKey = await crypto.subtle.importKey(
            "jwk", JWK_PRIVATE,
            { name: "RSA-OAEP", hash: "SHA-256" },
            false, ["decrypt"]);
        results.push(["import-jwk",
            pubKey.type === "public" && privKey.type === "private" &&
            pubKey.algorithm.name === "RSA-OAEP" &&
            pubKey.algorithm.modulusLength === 2048]);
    } catch (e) {
        results.push(["import-jwk", false]);
        return results;
    }

    // ── Round-trip: encrypt → decrypt ─────────────────────────────────
    {
        const plaintext = enc.encode("hello rsa-oaep");
        const ct = await crypto.subtle.encrypt({ name: "RSA-OAEP" }, pubKey, plaintext);
        const pt = await crypto.subtle.decrypt({ name: "RSA-OAEP" }, privKey, ct);
        results.push(["roundtrip-empty-label",
            ct.byteLength === 256 &&
            dec.decode(pt) === "hello rsa-oaep"]);
    }

    // ── Label binding ─────────────────────────────────────────────────
    {
        const plaintext = enc.encode("labeled message");
        const label = enc.encode("context-label");
        const ct = await crypto.subtle.encrypt({ name: "RSA-OAEP", label }, pubKey, plaintext);
        const pt = await crypto.subtle.decrypt({ name: "RSA-OAEP", label }, privKey, ct);
        results.push(["roundtrip-with-label", dec.decode(pt) === "labeled message"]);
        // Wrong label must reject.
        let rejected = false;
        try {
            await crypto.subtle.decrypt(
                { name: "RSA-OAEP", label: enc.encode("wrong-label") }, privKey, ct);
        } catch (_) { rejected = true; }
        results.push(["wrong-label-rejected", rejected]);
    }

    // ── Tampered-ciphertext rejection ─────────────────────────────────
    {
        const plaintext = enc.encode("secret");
        const ct = await crypto.subtle.encrypt({ name: "RSA-OAEP" }, pubKey, plaintext);
        const tampered = new Uint8Array(ct);
        tampered[0] ^= 0x01;
        let rejected = false;
        try {
            await crypto.subtle.decrypt({ name: "RSA-OAEP" }, privKey, tampered);
        } catch (_) { rejected = true; }
        results.push(["tampered-rejected", rejected]);
    }

    // ── Ciphertext is randomized (two encrypts differ) ───────────────
    {
        const plaintext = enc.encode("same plaintext");
        const ct1 = await crypto.subtle.encrypt({ name: "RSA-OAEP" }, pubKey, plaintext);
        const ct2 = await crypto.subtle.encrypt({ name: "RSA-OAEP" }, pubKey, plaintext);
        const a = new Uint8Array(ct1);
        const b = new Uint8Array(ct2);
        let differ = false;
        for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) { differ = true; break; }
        results.push(["randomized-ciphertext", differ]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
