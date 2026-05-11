// Tier-J consumer #28: PBKDF2 deriveBits over SHA-1/256/384/512.
//
// Validates E.8 extension to PBKDF2 key derivation. Real consumer use
// cases: password-hashing (CipherSweet, libsodium-fallbacks), Argon2-less
// envs, scrypt-adjacent KDFs, JOSE PBES2. Test vectors from RFC 6070
// (PBKDF2-HMAC-SHA-1) + RFC 7914 §11 (PBKDF2-HMAC-SHA-256).

function bytesToHex(buf) {
    const arr = buf instanceof ArrayBuffer ? new Uint8Array(buf) : buf;
    let hex = "";
    for (let i = 0; i < arr.length; i++) {
        hex += arr[i].toString(16).padStart(2, "0");
    }
    return hex;
}

async function importPbkdf2(password) {
    const enc = new TextEncoder();
    return crypto.subtle.importKey(
        "raw",
        typeof password === "string" ? enc.encode(password) : password,
        { name: "PBKDF2" },
        false,
        ["deriveBits"],
    );
}

async function selfTest() {
    const results = [];
    const enc = new TextEncoder();

    // ── RFC 6070 PBKDF2-HMAC-SHA-1 ────────────────────────────────────

    // 1. P="password", S="salt", c=1, dkLen=20
    //    DK = 0c60c80f961f0e71f3a9b524af6012062fe037a6
    {
        const key = await importPbkdf2("password");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-1", salt: enc.encode("salt"), iterations: 1 },
            key, 160,
        );
        results.push(["rfc6070-test1",
            dk instanceof ArrayBuffer && dk.byteLength === 20 &&
            bytesToHex(dk) === "0c60c80f961f0e71f3a9b524af6012062fe037a6"]);
    }

    // 2. c=2, same params
    //    DK = ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957
    {
        const key = await importPbkdf2("password");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-1", salt: enc.encode("salt"), iterations: 2 },
            key, 160,
        );
        results.push(["rfc6070-test2",
            bytesToHex(dk) === "ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957"]);
    }

    // 3. c=4096
    //    DK = 4b007901b765489abead49d926f721d065a429c1
    {
        const key = await importPbkdf2("password");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-1", salt: enc.encode("salt"), iterations: 4096 },
            key, 160,
        );
        results.push(["rfc6070-test3",
            bytesToHex(dk) === "4b007901b765489abead49d926f721d065a429c1"]);
    }

    // 4. Multi-block output (dkLen=25, longer than 20-byte SHA-1 block).
    //    P="passwordPASSWORDpassword" S="saltSALTsaltSALTsaltSALTsaltSALTsalt" c=4096
    //    DK = 3d2eec4fe41c849b80c8d83662c0e44a8b291a964cf2f07038
    {
        const key = await importPbkdf2("passwordPASSWORDpassword");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-1",
              salt: enc.encode("saltSALTsaltSALTsaltSALTsaltSALTsalt"), iterations: 4096 },
            key, 200,
        );
        results.push(["rfc6070-test5",
            dk.byteLength === 25 &&
            bytesToHex(dk) === "3d2eec4fe41c849b80c8d83662c0e44a8b291a964cf2f07038"]);
    }

    // ── RFC 7914 §11 PBKDF2-HMAC-SHA-256 ──────────────────────────────

    // 5. P="passwd", S="salt", c=1, dkLen=64
    {
        const key = await importPbkdf2("passwd");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-256", salt: enc.encode("salt"), iterations: 1 },
            key, 512,
        );
        results.push(["rfc7914-sha256-c1",
            dk.byteLength === 64 &&
            bytesToHex(dk) ===
            "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc49ca9cccf179b645991664b39d77ef317c71b845b1e30bd509112041d3a19783"]);
    }

    // ── SHA-384 / SHA-512 round-trips ─────────────────────────────────
    //
    // Vectors not standardized in a stable RFC across implementations;
    // assert structural properties: correct length, determinism, hash
    // isolation. Bun & rusty-bun-host both compute the same answer.

    // 6. SHA-384 derive 48 bytes is deterministic.
    {
        const k1 = await importPbkdf2("hunter2");
        const k2 = await importPbkdf2("hunter2");
        const a = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-384", salt: enc.encode("NaCl"), iterations: 1000 },
            k1, 384);
        const b = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-384", salt: enc.encode("NaCl"), iterations: 1000 },
            k2, 384);
        results.push(["sha384-deterministic",
            a.byteLength === 48 && bytesToHex(a) === bytesToHex(b)]);
    }

    // 7. SHA-512 derive 64 bytes; different hash → different output.
    {
        const k = await importPbkdf2("hunter2");
        const k2 = await importPbkdf2("hunter2");
        const sha384 = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-384", salt: enc.encode("NaCl"), iterations: 1000 },
            k, 384);
        const sha512 = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-512", salt: enc.encode("NaCl"), iterations: 1000 },
            k2, 512);
        results.push(["hash-isolation",
            sha384.byteLength === 48 && sha512.byteLength === 64 &&
            bytesToHex(sha384) !== bytesToHex(sha512).slice(0, 96)]);
    }

    // 8. Salt sensitivity — flipping one byte of salt changes output.
    {
        const k = await importPbkdf2("hunter2");
        const k2 = await importPbkdf2("hunter2");
        const a = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-256", salt: enc.encode("salt-A"), iterations: 100 },
            k, 256);
        const b = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-256", salt: enc.encode("salt-B"), iterations: 100 },
            k2, 256);
        results.push(["salt-sensitive", bytesToHex(a) !== bytesToHex(b)]);
    }

    // 9. Iteration sensitivity.
    {
        const k = await importPbkdf2("hunter2");
        const k2 = await importPbkdf2("hunter2");
        const a = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-256", salt: enc.encode("salt"), iterations: 100 },
            k, 256);
        const b = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-256", salt: enc.encode("salt"), iterations: 101 },
            k2, 256);
        results.push(["iteration-sensitive", bytesToHex(a) !== bytesToHex(b)]);
    }

    // 10. Output length truncation: dkLen=8 (one-byte truncation of a block).
    {
        const k = await importPbkdf2("password");
        const dk = await crypto.subtle.deriveBits(
            { name: "PBKDF2", hash: "SHA-1", salt: enc.encode("salt"), iterations: 1 },
            k, 64);
        results.push(["short-output",
            dk.byteLength === 8 &&
            bytesToHex(dk) === "0c60c80f961f0e71"]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
