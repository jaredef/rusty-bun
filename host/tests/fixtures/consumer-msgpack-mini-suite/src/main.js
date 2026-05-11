// Tier-J consumer #45: vendored msgpack-mini binary codec.
//
// Most-distinct remaining axis: binary-protocol parsing via
// Uint8Array + DataView + bit-level format-byte dispatch.
// Roundtrip-verifies the major MessagePack types and confirms
// wire-level byte-exact compatibility with a known fixture vector.

import msgpack, { encode, decode } from "msgpack-mini";

function bytesEq(a, b) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
}
function hex(u8) {
    let s = ""; for (const b of u8) s += b.toString(16).padStart(2, "0");
    return s;
}

async function selfTest() {
    const results = [];

    // 1. nil / true / false fixed bytes per spec.
    results.push(["nil",   hex(encode(null))  === "c0"]);
    results.push(["true",  hex(encode(true))  === "c3"]);
    results.push(["false", hex(encode(false)) === "c2"]);

    // 2. positive fixint: 0..127 emit as single byte equal to value.
    results.push(["positive-fixint",
        hex(encode(0))   === "00" &&
        hex(encode(127)) === "7f"]);

    // 3. negative fixint: -32..-1 emit single byte.
    results.push(["negative-fixint",
        hex(encode(-1))  === "ff" &&
        hex(encode(-32)) === "e0"]);

    // 4. uint8 prefix 0xcc.
    results.push(["uint8",
        hex(encode(255)) === "ccff"]);

    // 5. uint16 prefix 0xcd big-endian.
    results.push(["uint16",
        hex(encode(256))   === "cd0100" &&
        hex(encode(65535)) === "cdffff"]);

    // 6. int16 prefix 0xd1 — negative encoding.
    results.push(["int16",
        hex(encode(-1000)) === "d1fc18"]);

    // 7. fixstr prefix 0xa0|len + UTF-8.
    results.push(["fixstr",
        hex(encode("hi")) === "a26869"]);

    // 8. str8 prefix 0xd9.
    {
        const s32 = "x".repeat(32);
        const out = encode(s32);
        results.push(["str8", out[0] === 0xd9 && out[1] === 32 && out.length === 34]);
    }

    // 9. fixarray prefix 0x90|len.
    results.push(["fixarray",
        hex(encode([1, 2, 3])) === "93010203"]);

    // 10. fixmap prefix 0x80|len with string keys.
    {
        // Note: msgpack-mini uses Object.keys which preserves insertion order.
        const out = encode({ a: 1, b: 2 });
        results.push(["fixmap",
            hex(out) === "82a16101a16202"]);
    }

    // 11. Round-trip: nested object with mixed types.
    {
        const orig = {
            name: "alice",
            age: 30,
            active: true,
            tags: ["admin", "user"],
            scores: { math: 95, english: 88 },
            avatar: null,
        };
        const enc = encode(orig);
        const dec = decode(enc);
        results.push(["roundtrip-nested",
            dec.name === "alice" &&
            dec.age === 30 &&
            dec.active === true &&
            dec.tags[0] === "admin" && dec.tags[1] === "user" &&
            dec.scores.math === 95 && dec.scores.english === 88 &&
            dec.avatar === null]);
    }

    // 12. Round-trip: binary blob.
    {
        const bin = new Uint8Array([0xde, 0xad, 0xbe, 0xef, 0x00, 0xff]);
        const dec = decode(encode(bin));
        results.push(["roundtrip-binary",
            dec instanceof Uint8Array && bytesEq(dec, bin)]);
    }

    // 13. Round-trip: float64.
    {
        const out = decode(encode(3.14159265358979));
        results.push(["roundtrip-float", out === 3.14159265358979]);
    }

    // 14. Round-trip: negative int32 (boundary).
    {
        const v = -2000000000;
        results.push(["roundtrip-int32",
            decode(encode(v)) === v]);
    }

    // 15. Round-trip: string with multi-byte UTF-8.
    {
        const s = "héllo 世界 🌍";
        results.push(["roundtrip-utf8",
            decode(encode(s)) === s]);
    }

    // 16. Round-trip: empty containers.
    {
        results.push(["empty-array", JSON.stringify(decode(encode([]))) === "[]"]);
        results.push(["empty-object", JSON.stringify(decode(encode({}))) === "{}"]);
    }

    // 17. Decode-only: hand-crafted msgpack bytes for {"compact": true, "schema": 0}
    //     This is the canonical "Why use MessagePack?" example from the spec README.
    {
        const buf = new Uint8Array([
            0x82,                         // fixmap, 2 entries
            0xa7, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x63, 0x74,  // "compact"
            0xc3,                                              // true
            0xa6, 0x73, 0x63, 0x68, 0x65, 0x6d, 0x61,         // "schema"
            0x00,                                              // 0
        ]);
        const out = decode(buf);
        results.push(["spec-canonical",
            out.compact === true && out.schema === 0]);
    }

    // 18. Default-export shape.
    results.push(["default-export",
        msgpack.encode === encode && msgpack.decode === decode]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
