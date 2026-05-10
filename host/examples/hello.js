// Demo script for the rusty-bun-host CLI. Exercises several wired pilots
// from JS code running in the embedded rquickjs runtime.
//
// Run: rusty-bun-host host/examples/hello.js
//
// Expected output: each demo line either succeeds silently (assert) or
// prints the result via the host's eventual console binding (not yet
// wired; for now, the JS side throws on failure and exits 1).

// btoa / atob roundtrip
const greeting = "hello, world!";
const encoded = btoa(greeting);
const decoded = atob(encoded);
if (decoded !== greeting) {
    throw new Error("atob(btoa(x)) !== x");
}

// path.* manipulation
const p = "/usr/local/bin/node";
const dir = path.dirname(p);
const base = path.basename(p);
const ext = path.extname("script.test.ts");
if (dir !== "/usr/local/bin") throw new Error("dirname wrong: " + dir);
if (base !== "node") throw new Error("basename wrong: " + base);
if (ext !== ".ts") throw new Error("extname wrong: " + ext);

// crypto.randomUUID
const uuid = crypto.randomUUID();
if (uuid.length !== 36) throw new Error("uuid length: " + uuid.length);
if (uuid[14] !== "4") throw new Error("uuid version: " + uuid[14]);

// Compositional: chain pilots together
const cipher = btoa(p);
const recovered = atob(cipher);
const inferred = path.basename(recovered);
if (inferred !== "node") throw new Error("composition failed");

// If we reach here, everything passed. The JS evaluator returns the last
// expression's value, which we'll keep as a sentinel string.
"all wired pilots work from JS"
