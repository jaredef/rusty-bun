// Tier-J consumer #35: ECDH P-256 deriveBits.
//
// JWA ECDH-ES content-key derivation (JWE), TLS ECDHE handshakes, Noise
// Protocol IK pattern, WebAuthn attestation. ECDH is deterministic given
// the two keys, so byte-equal shared-secret IS the cross-engine signal —
// stronger than the randomized-signature surfaces above.

const ALICE = {
    kty: "EC", crv: "P-256",
    x: "qtrKZLjMvpXPWNIOJUcb64NsDqQKRZPG3xTL4OwmXf4",
    y: "GQDO2J4V0WLtrI3vN_Yzy-eaWcPDW3ubjEkGPDXZJqU",
    d: "XqBnQt6Dkwgr5tH4tfR5VZP9esDwK0F76888kVcRUL0",
};
const BOB = {
    kty: "EC", crv: "P-256",
    x: "6r_NMj-JV1ApgEnKDLFWKbByZYaMe-JDBFWJ19R30LE",
    y: "AK0IOI2gbCFyM9wY-8L55pt0PZ4VvbPugvzJHTSxf6E",
    d: "DoBDkQLAdNGR9hP2nkX-O3MKLwjSPVQpyVsUMUJthds",
};
const A_PUB = { kty: ALICE.kty, crv: ALICE.crv, x: ALICE.x, y: ALICE.y };
const B_PUB = { kty: BOB.kty,   crv: BOB.crv,   x: BOB.x,   y: BOB.y };

function bytesToHex(buf) {
    const arr = buf instanceof ArrayBuffer ? new Uint8Array(buf) : buf;
    let hex = "";
    for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, "0");
    return hex;
}

async function selfTest() {
    const results = [];

    const aPriv = await crypto.subtle.importKey(
        "jwk", ALICE, { name: "ECDH", namedCurve: "P-256" }, false, ["deriveBits"]);
    const aPub  = await crypto.subtle.importKey(
        "jwk", A_PUB, { name: "ECDH", namedCurve: "P-256" }, false, []);
    const bPriv = await crypto.subtle.importKey(
        "jwk", BOB, { name: "ECDH", namedCurve: "P-256" }, false, ["deriveBits"]);
    const bPub  = await crypto.subtle.importKey(
        "jwk", B_PUB, { name: "ECDH", namedCurve: "P-256" }, false, []);
    results.push(["import", aPriv.type === "private" && bPub.type === "public" &&
                            aPriv.algorithm.name === "ECDH"]);

    // DH property: dA·QB = dB·QA. The x-coordinate is the shared secret.
    const fromA = await crypto.subtle.deriveBits({ name: "ECDH", public: bPub }, aPriv, 256);
    const fromB = await crypto.subtle.deriveBits({ name: "ECDH", public: aPub }, bPriv, 256);
    results.push(["diffie-hellman-property",
        fromA.byteLength === 32 && bytesToHex(fromA) === bytesToHex(fromB)]);

    // Truncation: requesting 128 bits returns the first 16 bytes.
    const truncated = await crypto.subtle.deriveBits({ name: "ECDH", public: bPub }, aPriv, 128);
    results.push(["truncation",
        truncated.byteLength === 16 &&
        bytesToHex(truncated) === bytesToHex(new Uint8Array(fromA).slice(0, 16))]);

    // Different peer → different shared secret.
    // Self-DH: deriving with my own pub key vs peer's pub key produces
    // different output (sanity check that the peer key actually flows through).
    const selfDh = await crypto.subtle.deriveBits({ name: "ECDH", public: aPub }, aPriv, 256);
    results.push(["peer-key-flows-through",
        bytesToHex(selfDh) !== bytesToHex(fromA)]);

    // Determinism: re-deriving gives the same shared secret.
    const again = await crypto.subtle.deriveBits({ name: "ECDH", public: bPub }, aPriv, 256);
    results.push(["deterministic", bytesToHex(again) === bytesToHex(fromA)]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
