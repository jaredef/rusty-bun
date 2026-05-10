// Middleware: SHA-256 sign the canonical-JSON serialization of the
// normalized request. Real-world API signing pattern.
import { canonicalJson } from "../lib/canonical.js";

function bytesToHex(bytes) {
    const arr = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes;
    let hex = "";
    for (let i = 0; i < arr.length; i++) {
        hex += arr[i].toString(16).padStart(2, "0");
    }
    return hex;
}

export function sign(next) {
    return async function signingHandler(ctx) {
        const canonical = canonicalJson(ctx.normalized);
        const enc = new TextEncoder();
        const digest = await crypto.subtle.digest("SHA-256", enc.encode(canonical));
        ctx.signature = bytesToHex(digest);
        ctx.canonical = canonical;
        return next(ctx);
    };
}
