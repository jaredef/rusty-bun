// Bun-portable differential script. Exercises only spec-portable surfaces
// that both Bun and rusty-bun-host implement equivalently. Produces a
// deterministic line-per-test output that the differential runner diffs
// across the two runtimes.
//
// Sub-criterion 5: real consumer code's spec-portable subset runs
// identically on both runtimes. Each line is `name=value`. Equality of
// the full output across runtimes = differential pass.

const out = [];

function record(name, value) {
    out.push(name + "=" + JSON.stringify(value));
}

// ─── URL ──────────────────────────────────────────────────────────
const u = new URL("https://example.com:8443/api/v1/users?limit=10&offset=20#frag");
record("url.protocol", u.protocol);
record("url.hostname", u.hostname);
record("url.port", u.port);
record("url.pathname", u.pathname);
record("url.search", u.search);
record("url.hash", u.hash);
record("url.origin", u.origin);
record("url.href", u.href);

const rel = new URL("./other?x=1", "https://example.com/foo/bar/");
record("url.relative.href", rel.href);

// ─── URLSearchParams ──────────────────────────────────────────────
const p = new URLSearchParams();
p.append("a", "1");
p.append("b", "hello world");
p.append("a", "2");
record("usp.toString", p.toString());
record("usp.getAll.a", p.getAll("a"));
record("usp.has.a", p.has("a"));
record("usp.has.z", p.has("z"));

const sorted = new URLSearchParams("c=1&a=2&b=3");
sorted.sort();
record("usp.sorted", sorted.toString());

// ─── structuredClone ──────────────────────────────────────────────
const original = {
    n: 42,
    s: "hello",
    arr: [1, [2, 3], { k: "v" }],
    map: new Map([["x", 1], ["y", 2]]),
    set: new Set([1, 2, 3]),
    date: new Date(0),  // 1970-01-01 epoch
};
const cloned = structuredClone(original);
record("clone.n", cloned.n);
record("clone.arr.deep", cloned.arr[2].k);
record("clone.map.x", cloned.map.get("x"));
record("clone.set.size", cloned.set.size);
record("clone.date.iso", cloned.date.toISOString());
record("clone.independence",
    (cloned.arr[1][0] = 99, original.arr[1][0] === 2 && cloned.arr[1][0] === 99));

// ─── Buffer ───────────────────────────────────────────────────────
// Bun.Node-Buffer has instance .toString("encoding"); rusty-bun-host's
// Buffer is a namespace of static helpers. Use only what's portable
// between them: Buffer.byteLength + literal Buffer.from-based length.
const buf = Buffer.from("hello world");
record("buffer.length", buf.length);
record("buffer.byteLength", Buffer.byteLength("héllo"));

// ─── TextEncoder / TextDecoder ────────────────────────────────────
const enc = new TextEncoder();
const dec = new TextDecoder();
record("text.encoding", enc.encoding);
record("text.roundtrip", dec.decode(enc.encode("héllo, мир! 🌍")));

// ─── atob / btoa ──────────────────────────────────────────────────
record("btoa", btoa("hello"));
record("atob", atob("aGVsbG8="));

// ─── crypto.randomUUID format ─────────────────────────────────────
const uuid = crypto.randomUUID();
record("uuid.length", uuid.length);
record("uuid.version", uuid[14]);
record("uuid.format", /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(uuid));

// ─── Pure JS: Date / Map / Set / JSON ─────────────────────────────
record("date.epoch.iso", new Date(0).toISOString());
record("json.stringify", JSON.stringify({ a: 1, b: [2, 3] }));
record("json.parse.roundtrip", JSON.parse('{"k":42}').k);

// ─── Headers ──────────────────────────────────────────────────────
const h = new Headers();
h.set("Content-Type", "application/json");
h.append("X-Custom", "value");
record("headers.get.lowercase", h.get("content-type"));
record("headers.has.uppercase", h.has("X-Custom"));

// ─── Output: emit deterministic newline-separated lines ──────────
const result = out.join("\n");
if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    // Bun runtime path.
    process.stdout.write(result + "\n");
} else {
    // rusty-bun-host path: store on globalThis for Rust-side capture.
    globalThis.__esmResult = result;
}
