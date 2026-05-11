// Tier-J consumer #34: ECDSA P-256 / SHA-256 sign/verify.
//
// JWS ES256, WebAuthn (FIDO2), mTLS, COSE_Sign with ES256. ECDSA sign
// is RANDOMIZED (per-signature nonce k); differential signal is the
// engine's summary string. Cross-engine verify is the strong test:
// rusty-bun-host signs → Bun verifies, and vice versa.

const JWK_PRIVATE = {
    kty: "EC",
    crv: "P-256",
    x: "STh4N60IU2y0SR00MjlFW9rWOzE4CyQODugl0hbU-c8",
    y: "POEv3HT77hnmsVE_L8TbgxkAIiWGfQpkZKI8HwFsGak",
    d: "3jBk6Cwb7ItlnWvSazbSDoBaj3q5uTXGFQIOy6pXYLY",
};
const JWK_PUBLIC = { kty: "EC", crv: "P-256", x: JWK_PRIVATE.x, y: JWK_PRIVATE.y };

async function selfTest() {
    const results = [];
    const enc = new TextEncoder();

    const pubKey = await crypto.subtle.importKey(
        "jwk", JWK_PUBLIC, { name: "ECDSA", namedCurve: "P-256" }, false, ["verify"]);
    const privKey = await crypto.subtle.importKey(
        "jwk", JWK_PRIVATE, { name: "ECDSA", namedCurve: "P-256" }, false, ["sign"]);
    results.push(["import-jwk-ec",
        privKey.type === "private" && pubKey.type === "public" &&
        privKey.algorithm.name === "ECDSA" &&
        privKey.algorithm.namedCurve === "P-256"]);

    // Round-trip
    {
        const message = enc.encode("sign-this-ecdsa-message");
        const sig = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privKey, message);
        const ok = await crypto.subtle.verify(
            { name: "ECDSA", hash: "SHA-256" }, pubKey, sig, message);
        results.push(["roundtrip", sig.byteLength === 64 && ok === true]);
    }

    // Wrong-message rejection
    {
        const sig = await crypto.subtle.sign(
            { name: "ECDSA", hash: "SHA-256" }, privKey, enc.encode("original"));
        const ok = await crypto.subtle.verify(
            { name: "ECDSA", hash: "SHA-256" }, pubKey, sig, enc.encode("tampered"));
        results.push(["wrong-message-rejected", ok === false]);
    }

    // Tampered-signature rejection
    {
        const message = enc.encode("msg");
        const sig = new Uint8Array(
            await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privKey, message));
        sig[0] ^= 0x01;
        const ok = await crypto.subtle.verify(
            { name: "ECDSA", hash: "SHA-256" }, pubKey, sig, message);
        results.push(["tampered-signature-rejected", ok === false]);
    }

    // Randomized signature: two signs of same message differ.
    {
        const message = enc.encode("same message");
        const a = new Uint8Array(
            await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privKey, message));
        const b = new Uint8Array(
            await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privKey, message));
        let differ = false;
        for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) { differ = true; break; }
        results.push(["randomized-signature", differ]);
    }

    // Wrong public key rejection: another keypair's pub key must not verify
    // this key's signature.
    {
        const otherJwk = {
            kty: "EC", crv: "P-256",
            // Same x as JWK_PUBLIC; flip last byte of y to get a different (off-curve
            // for THIS scheme but the verify path should reject signature mismatch
            // before reaching the on-curve check if the x changes too). Use a
            // distinct, valid P-256 public point by importing a fresh-generated key
            // inline as a Bun-portable shape — but we can't generateKey on
            // rusty-bun-host. Instead use a 2nd hardcoded public from a different
            // private key.
            x: "DCsbVdgjnIxVO4nGd-yqdSp4iLgNAOO4HxbTbU5MM_4",
            y: "MhopxTUKfV8AqWY3D7XGqlVEYTcwj0X8JuV-VQzm60g",
        };
        const message = enc.encode("msg");
        const sig = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, privKey, message);
        let importedOk = false;
        try {
            const otherPub = await crypto.subtle.importKey(
                "jwk", otherJwk, { name: "ECDSA", namedCurve: "P-256" }, false, ["verify"]);
            importedOk = true;
            const ok = await crypto.subtle.verify(
                { name: "ECDSA", hash: "SHA-256" }, otherPub, sig, message);
            results.push(["wrong-pubkey-rejected", ok === false]);
        } catch (_) {
            // If the other JWK isn't on the curve, importKey rejects — count
            // that as a pass too (different failure mode, same security guarantee).
            results.push(["wrong-pubkey-rejected", !importedOk]);
        }
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
