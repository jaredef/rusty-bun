// Tier-J consumer #10: config-merger (spec-first M9 authoring).
//
// In-basin axes per probes 2026-05-10:
//   - Promise.withResolvers (ES2024)
//   - Array.prototype.toSorted / .toReversed / .toSpliced / .with (ES2023)
//   - Object.groupBy (ES2024)
//   - structuredClone on TypedArrays (Uint8Array)
//   - Deep object spread for nested config merging
//
// Stays clear of E.10 (Set.union/intersection/difference) and other
// recorded boundaries.

const SOURCES = [
    { name: "default", priority: 0, settings: { theme: "auto", maxRetries: 3 } },
    { name: "user",    priority: 2, settings: { theme: "dark", debug: true } },
    { name: "env",     priority: 1, settings: { maxRetries: 5, apiUrl: "https://example.com" } },
    { name: "cli",     priority: 3, settings: { debug: false } },
];

function mergeConfigs(sources) {
    // Higher priority overrides lower — use toSorted (immutable) so the
    // input array isn't mutated.
    const ordered = sources.toSorted((a, b) => a.priority - b.priority);
    return ordered.reduce((acc, src) => ({ ...acc, ...src.settings }), {});
}

async function deferredLoader(value, ms) {
    // Promise.withResolvers — modern deferred pattern.
    const { promise, resolve, reject } = Promise.withResolvers();
    if (value === null) reject(new Error("null value"));
    else setTimeout(() => resolve(value), ms);
    return promise;
}

async function selfTest() {
    const results = [];

    // 1. Array.prototype.toSorted — immutable, returns new array.
    const arr = [3, 1, 4, 1, 5];
    const sortedNew = arr.toSorted((a, b) => a - b);
    results.push(["array-toSorted",
        JSON.stringify(arr) === "[3,1,4,1,5]" &&
        JSON.stringify(sortedNew) === "[1,1,3,4,5]"]);

    // 2. Array.prototype.toReversed.
    const rev = [1, 2, 3].toReversed();
    results.push(["array-toReversed", JSON.stringify(rev) === "[3,2,1]"]);

    // 3. Array.prototype.with — replace at index, returning new array.
    const replaced = ["a", "b", "c"].with(1, "B");
    results.push(["array-with", JSON.stringify(replaced) === '["a","B","c"]']);

    // 4. Array.prototype.toSpliced — immutable splice.
    const spliced = [1, 2, 3, 4, 5].toSpliced(1, 2, 99, 100, 101);
    results.push(["array-toSpliced", JSON.stringify(spliced) === "[1,99,100,101,4,5]"]);

    // 5. Config merge using toSorted + reduce + spread.
    const merged = mergeConfigs(SOURCES);
    results.push(["config-merge-priority",
        merged.theme === "dark" &&            // user (priority 2) overrides default
        merged.maxRetries === 5 &&            // env (priority 1) overrides default
        merged.debug === false &&             // cli (priority 3) overrides user
        merged.apiUrl === "https://example.com"]);

    // 6. Object.groupBy — group records by a computed key.
    const records = [
        { kind: "error", count: 1 },
        { kind: "info", count: 10 },
        { kind: "error", count: 2 },
        { kind: "warn", count: 5 },
        { kind: "info", count: 7 },
    ];
    const grouped = Object.groupBy(records, (r) => r.kind);
    results.push(["object-groupBy",
        grouped.error.length === 2 &&
        grouped.info.length === 2 &&
        grouped.warn.length === 1]);

    // 7. structuredClone on Uint8Array.
    const buf = new Uint8Array([10, 20, 30, 40]);
    const cloned = structuredClone(buf);
    cloned[0] = 99;
    results.push(["structuredClone-TypedArray",
        buf[0] === 10 && cloned[0] === 99 &&
        cloned.constructor === Uint8Array && cloned.length === 4]);

    // 8. Promise.withResolvers — deferred resolution.
    const val = await deferredLoader("loaded", 0);
    results.push(["promise-withResolvers", val === "loaded"]);

    // 9. Promise.withResolvers — deferred rejection path.
    let caught = null;
    try { await deferredLoader(null, 0); }
    catch (e) { caught = e; }
    results.push(["promise-withResolvers-reject",
        caught !== null && caught.message === "null value"]);

    // 10. Deep object spread for nested config merging.
    const a = { server: { host: "localhost", port: 80 }, debug: false };
    const b = { server: { port: 8080 } };  // partial override at server.port
    // Naive spread is shallow — for nested, recursive merge needed.
    // Most consumer code uses a custom helper or library; here we test
    // the shallow shape explicitly.
    const shallow = { ...a, ...b };
    results.push(["object-spread-shallow",
        shallow.server.host === undefined &&   // shallow overwrites
        shallow.server.port === 8080 &&
        shallow.debug === false]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    process.stdout.write(summary + "\n");
} else {
    globalThis.__esmResult = summary;
}
