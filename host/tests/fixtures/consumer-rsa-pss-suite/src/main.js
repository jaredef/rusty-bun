// Tier-J consumer #33: RSA-PSS sign/verify.
//
// JWS PS256/PS384/PS512, modern JWT signing, COSE_Sign with rsa-pss.
// PSS sign is RANDOMIZED (when saltLength > 0); the differential is
// the engine's summary string, not byte-equal signatures.

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

    const pubKey = await crypto.subtle.importKey(
        "jwk", JWK_PUBLIC, { name: "RSA-PSS", hash: "SHA-256" }, false, ["verify"]);
    const privKey = await crypto.subtle.importKey(
        "jwk", JWK_PRIVATE, { name: "RSA-PSS", hash: "SHA-256" }, false, ["sign"]);
    results.push(["import-jwk",
        privKey.type === "private" && pubKey.type === "public" &&
        privKey.algorithm.name === "RSA-PSS"]);

    // 1. Sign → verify round-trip with saltLength=32 (default for SHA-256).
    {
        const message = enc.encode("sign this");
        const sig = await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privKey, message);
        const ok = await crypto.subtle.verify({ name: "RSA-PSS", saltLength: 32 }, pubKey, sig, message);
        results.push(["roundtrip", sig.byteLength === 256 && ok === true]);
    }

    // 2. Wrong-message rejection.
    {
        const sig = await crypto.subtle.sign(
            { name: "RSA-PSS", saltLength: 32 }, privKey, enc.encode("original"));
        const ok = await crypto.subtle.verify(
            { name: "RSA-PSS", saltLength: 32 }, pubKey, sig, enc.encode("tampered"));
        results.push(["wrong-message-rejected", ok === false]);
    }

    // 3. Tampered-signature rejection.
    {
        const message = enc.encode("msg");
        const sig = new Uint8Array(
            await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privKey, message));
        sig[10] ^= 0x01;
        const ok = await crypto.subtle.verify(
            { name: "RSA-PSS", saltLength: 32 }, pubKey, sig, message);
        results.push(["tampered-signature-rejected", ok === false]);
    }

    // 4. Different signatures for same message (PSS is randomized).
    {
        const message = enc.encode("same message");
        const a = new Uint8Array(
            await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privKey, message));
        const b = new Uint8Array(
            await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 32 }, privKey, message));
        let differ = false;
        for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) { differ = true; break; }
        results.push(["randomized-signature", differ]);
    }

    // 5. Deterministic mode (saltLength=0).
    {
        const message = enc.encode("deterministic");
        const a = new Uint8Array(
            await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 0 }, privKey, message));
        const b = new Uint8Array(
            await crypto.subtle.sign({ name: "RSA-PSS", saltLength: 0 }, privKey, message));
        let equal = a.length === b.length;
        for (let i = 0; equal && i < a.length; i++) if (a[i] !== b[i]) equal = false;
        const ok = await crypto.subtle.verify(
            { name: "RSA-PSS", saltLength: 0 }, pubKey, a, message);
        results.push(["deterministic-saltlen-0", equal && ok === true]);
    }

    // 6. Salt-length sensitivity: signing with sLen=32 then verifying with
    //    sLen=64 must reject (the verifier reads the wrong salt span).
    {
        const message = enc.encode("salt-binding");
        const sig = await crypto.subtle.sign(
            { name: "RSA-PSS", saltLength: 32 }, privKey, message);
        const ok = await crypto.subtle.verify(
            { name: "RSA-PSS", saltLength: 64 }, pubKey, sig, message);
        results.push(["salt-length-bound", ok === false]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
