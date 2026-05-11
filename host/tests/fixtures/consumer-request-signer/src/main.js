// Tier-J consumer #3: middleware-composed request signer.
//
// Bun-portable from inception (M8 anchor: every plank built plumb).
// Exercises shapes the prior fixtures didn't:
//   - Middleware composition: validate → augment → sign → terminal
//   - Deep ESM dep graph: 7 modules across src/, lib/, middleware/
//   - crypto.subtle.digest("SHA-256")
//   - Async iteration of ReadableStream (for-await-of)
//   - TextEncoder
//   - Canonical-JSON serialization for deterministic signing
//
// Six self-test cases verify the chain end-to-end and produce a
// deterministic "passed/total" line.

import { compose } from "../lib/compose.js";
import { makePayloadStream } from "../lib/payload-stream.js";
import { validate } from "../middleware/validate.js";
import { augment } from "../middleware/augment.js";
import { sign } from "../middleware/sign.js";

// Terminal handler — the last in the chain. Receives the fully-augmented
// + signed context and produces the response object.
async function terminal(ctx) {
    return {
        ok: true,
        user: ctx.normalized.user,
        action: ctx.normalized.action,
        signature: ctx.signature,
        canonical: ctx.canonical,
    };
}

const handler = compose(validate, augment, sign)(terminal);

async function selfTest() {
    const results = [];

    // 1. Happy path: valid payload yields a signed response.
    const r1 = await handler({ payload: { user: "Alice", action: "read" } });
    results.push(["happy-path", r1.ok && r1.user === "alice" && r1.action === "READ" &&
        r1.signature.length === 64]);

    // 2. Determinism: same input produces same signature on each call.
    const a = await handler({ payload: { user: "Bob", action: "write" } });
    const b = await handler({ payload: { user: "Bob", action: "write" } });
    results.push(["determinism", a.signature === b.signature]);

    // 3. Canonicalization: key order in payload doesn't affect signature.
    const c1 = await handler({ payload: { user: "Carol", action: "delete" } });
    const c2 = await handler({ payload: { action: "delete", user: "Carol" } });
    results.push(["canonical-order-invariant", c1.signature === c2.signature]);

    // 4. Validation: missing field throws.
    let caught = null;
    try { await handler({ payload: { user: "Dave" } }); }
    catch (e) { caught = e; }
    results.push(["validate-missing-field", caught !== null && /action required/.test(caught.message)]);

    // 5. Async stream iteration: process a stream of payloads.
    const payloads = [
        { user: "Eve", action: "read" },
        { user: "Frank", action: "write" },
        { user: "Grace", action: "exec" },
    ];
    const responses = [];
    const stream = makePayloadStream(payloads);
    for await (const p of stream) {
        responses.push(await handler({ payload: p }));
    }
    results.push(["stream-iter", responses.length === 3 && responses.every((r) => r.ok)]);

    // 6. Cross-pilot: serialize all responses with canonical JSON, hash
    // the hash, verify length and hex shape (composes signing + Buffer
    // when available, but stays portable using only TextEncoder + crypto).
    const allSigsCanonical = JSON.stringify(responses.map((r) => r.signature));
    const enc = new TextEncoder();
    const digest = await crypto.subtle.digest("SHA-256", enc.encode(allSigsCanonical));
    const arr = new Uint8Array(digest);
    let hex = "";
    for (let i = 0; i < arr.length; i++) hex += arr[i].toString(16).padStart(2, "0");
    results.push(["compose-digest-of-digests", hex.length === 64 && /^[0-9a-f]+$/.test(hex)]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
