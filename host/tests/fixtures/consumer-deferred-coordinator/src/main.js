// Tier-J consumer #13: deferred coordination (spec-first M9 authoring).
//
// In-basin axes per probes 2026-05-10. Picks shapes not yet exercised:
//   - Top-level await at module top-level for module setup
//   - Async generator with .return() early-exit (sync .return() was tested
//     in consumer-sequence-id; this is the async variant)
//   - WeakMap tracking promise-to-source associations
//   - Object.hasOwn (ES2022) — safer alternative to hasOwnProperty
//   - Array.prototype.at and String.prototype.at with negative indexing
//   - Promise.allSettled with mixed fulfilled+rejected outcomes
//   - try/finally cleanup paths driven by async-iter .return()

// Top-level await — module-level state set up before any export.
const moduleSetupValue = await Promise.resolve("setup-done");

// Source registry: who created each promise. WeakMap keyed by promise.
const sourceRegistry = new WeakMap();

function trackedPromise(source, value, shouldFail) {
    const p = shouldFail
        ? Promise.reject(new Error("source:" + source + " failed"))
        : Promise.resolve({ source, value });
    sourceRegistry.set(p, source);
    return p;
}

async function* drainPromises(promises) {
    for (const p of promises) {
        try {
            yield await p;
        } catch (e) {
            yield { error: true, message: e.message };
        }
    }
}

async function selfTest() {
    const results = [];

    // 1. Top-level await: moduleSetupValue is "setup-done".
    results.push(["top-level-await", moduleSetupValue === "setup-done"]);

    // 2. Promise.allSettled with mixed outcomes — count statuses.
    const settled = await Promise.allSettled([
        trackedPromise("alpha", 1, false),
        trackedPromise("beta", 2, true),    // rejects
        trackedPromise("gamma", 3, false),
        trackedPromise("delta", 4, true),   // rejects
    ]);
    const fulfilled = settled.filter((s) => s.status === "fulfilled").length;
    const rejected = settled.filter((s) => s.status === "rejected").length;
    results.push(["promise-allSettled-mixed",
        fulfilled === 2 && rejected === 2 &&
        settled[0].value.source === "alpha" &&
        settled[1].reason.message === "source:beta failed"]);

    // 3. Async generator drain with .return() early-exit.
    const promises = [
        trackedPromise("p1", 10, false),
        trackedPromise("p2", 20, false),
        trackedPromise("p3", 30, false),
        trackedPromise("p4", 40, false),
    ];
    const gen = drainPromises(promises);
    const collected = [];
    let cleanupSeen = false;
    for await (const result of gen) {
        collected.push(result.value);
        if (collected.length >= 2) {
            // Early-exit triggers async generator's .return().
            const ret = await gen.return("EARLY");
            cleanupSeen = ret.done === true && ret.value === "EARLY";
            break;
        }
    }
    results.push(["async-gen-return-early",
        collected.length === 2 && collected[0] === 10 && collected[1] === 20 &&
        cleanupSeen]);

    // 4. Object.hasOwn (ES2022) vs hasOwnProperty.
    const obj = { x: 1, y: undefined };
    const proto = Object.create(obj);
    results.push(["object-hasOwn",
        Object.hasOwn(obj, "x") === true &&
        Object.hasOwn(obj, "y") === true &&   // undefined value still owned
        Object.hasOwn(obj, "z") === false &&
        Object.hasOwn(proto, "x") === false]);  // inherited, not own

    // 5. Array.prototype.at with negative indexing.
    const arr = [10, 20, 30, 40, 50];
    results.push(["array-at-negative",
        arr.at(-1) === 50 && arr.at(-2) === 40 && arr.at(0) === 10 &&
        arr.at(-100) === undefined]);

    // 6. String.prototype.at with negative indexing.
    const s = "hello";
    results.push(["string-at-negative",
        s.at(-1) === "o" && s.at(-5) === "h" && s.at(-100) === undefined]);

    // 7. WeakMap source-tracking — the promises retain their source tags.
    const tagged = trackedPromise("ZZ", 99, false);
    results.push(["weakmap-promise-source",
        sourceRegistry.get(tagged) === "ZZ"]);

    // 8. try/finally cleanup driven by async-iter — finally block fires
    // even when caller breaks out early via .return().
    let finallyFired = false;
    async function* withFinally() {
        try {
            yield 1;
            yield 2;
            yield 3;
        } finally {
            finallyFired = true;
        }
    }
    const g2 = withFinally();
    await g2.next();
    await g2.return();
    results.push(["asyncgen-finally-on-return", finallyFired]);

    // 9. Promise.allSettled with empty array — resolves to empty array.
    const emptySettled = await Promise.allSettled([]);
    results.push(["allSettled-empty", Array.isArray(emptySettled) && emptySettled.length === 0]);

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
