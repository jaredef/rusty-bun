// Tier-J consumer #8: task pipeline (spec-first M9 authoring).
//
// In-basin axes per Doc 709 P1 probes:
//   - WeakMap (confirmed present 2026-05-10)
//   - Symbol.asyncIterator (confirmed present)
//   - Generator delegation yield* (in-basin per prior fixtures' async-gen)
//   - Function.prototype.bind for partial application
//   - Async iteration composition over user-defined classes
//   - WeakSet for membership-by-identity
//
// Stays clear of E.7 (WeakRef/FinalizationRegistry) and E.8 (subtle.importKey/sign).

import { Pipeline, concatPipelines } from "../lib/pipeline.js";
import { memoize } from "../lib/cache.js";

async function selfTest() {
    const results = [];

    // 1. Symbol.asyncIterator on a user-defined class drives for-await-of.
    const seen = [];
    const p1 = new Pipeline(["a", "b", "c"]).onItem((x) => seen.push("hook:" + x));
    const collected = [];
    for await (const item of p1) collected.push(item);
    results.push(["async-iterator-class",
        collected.join(",") === "a,b,c" &&
        seen.join(",") === "hook:a,hook:b,hook:c"]);

    // 2. Generator delegation via yield*.
    const left = new Pipeline([1, 2]);
    const right = new Pipeline([3, 4, 5]);
    const merged = [];
    for await (const v of concatPipelines(left, right)) merged.push(v);
    results.push(["yield-delegation", merged.join(",") === "1,2,3,4,5"]);

    // 3. WeakMap memoization keyed by object identity.
    const k1 = { id: 1 };
    const k2 = { id: 2 };
    let computeCount = 0;
    const compute = memoize((key) => {
        computeCount++;
        return key.id * 10;
    });
    const a = compute(k1);
    const b = compute(k1);  // cache hit, no recompute
    const c = compute(k2);
    results.push(["weakmap-memoize",
        a === 10 && b === 10 && c === 20 && computeCount === 2]);

    // 4. Function.prototype.bind partial application.
    function fmt(prefix, sep, value) {
        return prefix + sep + value;
    }
    const tagged = fmt.bind(null, "[INFO]", " - ");
    const out = tagged("hello") + "|" + tagged("world");
    results.push(["bind-partial",
        out === "[INFO] - hello|[INFO] - world"]);

    // 5. WeakSet membership-by-identity.
    const ws = new WeakSet();
    const obj1 = { name: "x" };
    const obj2 = { name: "y" };
    ws.add(obj1);
    results.push(["weakset",
        ws.has(obj1) === true && ws.has(obj2) === false]);

    // 6. Async iteration composition — pipeline of pipelines, collected via
    // an outer async generator.
    async function* doubledItems(pipeline) {
        for await (const item of pipeline) {
            yield typeof item === "number" ? item * 2 : item + item;
        }
    }
    const doubled = [];
    for await (const v of doubledItems(new Pipeline([1, 2, 3]))) doubled.push(v);
    results.push(["pipeline-composition", doubled.join(",") === "2,4,6"]);

    // 7. yield* + bind composed: build a sequence of formatted strings via
    // bound formatter applied to a yield-delegated stream.
    const tag = fmt.bind(null, "T", ":");
    const formatted = [];
    for await (const v of concatPipelines(new Pipeline(["a"]), new Pipeline(["b", "c"]))) {
        formatted.push(tag(v));
    }
    results.push(["yield-with-bind",
        formatted.join("|") === "T:a|T:b|T:c"]);

    // 8. WeakMap doesn't allow primitive keys — confirm the TypeError.
    const w = new WeakMap();
    let caught = null;
    try { w.set("string-key", 1); } catch (e) { caught = e; }
    results.push(["weakmap-primitive-rejects",
        caught instanceof TypeError]);

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
