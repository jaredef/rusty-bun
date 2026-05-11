// Tier-J consumer #58: node:util. Tier-Π3.10 closure round.
//
// node:util is the swiss-army-knife of node-builtins. The fixture
// exercises the load-bearing pieces real consumers depend on:
// promisify (canonical for legacy callback-style code), callbackify,
// format (logger-internal sprintf), inspect (debug repr), types
// predicates, and isDeepStrictEqual (test frameworks).

import { promisify, callbackify, format, inspect, isDeepStrictEqual, types } from "node:util";

async function selfTest() {
    const results = [];

    // 1. promisify wraps a callback-style fn into a Promise-returning fn.
    function legacyAdd(a, b, cb) { cb(null, a + b); }
    const addAsync = promisify(legacyAdd);
    const sum = await addAsync(2, 3);
    results.push(["promisify-basic", sum === 5]);

    // 2. promisify rejects on error-first callback.
    function legacyFail(cb) { cb(new Error("legacy fail")); }
    const failAsync = promisify(legacyFail);
    let caught = null;
    try { await failAsync(); } catch (e) { caught = e.message; }
    results.push(["promisify-rejects-on-error", caught === "legacy fail"]);

    // 3. callbackify wraps a Promise-returning fn into callback-style.
    async function modernDouble(x) { return x * 2; }
    const doubleCb = callbackify(modernDouble);
    const cb3 = await new Promise(resolve => {
        doubleCb(7, (err, v) => resolve({ err, v }));
    });
    results.push(["callbackify-basic", cb3.err === null && cb3.v === 14]);

    // 4. format with %s and %d.
    results.push(["format-basic",
        format("hello %s, you are %d", "world", 42) === "hello world, you are 42"]);

    // 5. format with %% literal.
    results.push(["format-percent-literal",
        format("100%% done in %ds", 3) === "100% done in 3s"]);

    // 6. inspect renders Arrays, Maps, Sets, plain objects.
    const inspectOk =
        inspect([1, 2, 3]) === "[ 1, 2, 3 ]" &&
        inspect({}) === "{}" &&
        inspect(new Map([["k", "v"]])).startsWith("Map(1)") &&
        inspect(new Set([1, 2])).startsWith("Set(2)");
    results.push(["inspect-common-shapes", inspectOk]);

    // 7. types.isXxx predicates.
    const typesOk =
        types.isPromise(Promise.resolve()) &&
        types.isDate(new Date()) &&
        types.isRegExp(/x/) &&
        types.isMap(new Map()) &&
        types.isSet(new Set()) &&
        types.isUint8Array(new Uint8Array()) &&
        types.isArrayBuffer(new ArrayBuffer(0)) &&
        !types.isDate({}) &&
        !types.isUint8Array([]);
    results.push(["types-predicates", typesOk]);

    // 8. isDeepStrictEqual handles nested + Map/Set + arrays.
    const deepOk =
        isDeepStrictEqual({ a: [1, 2, { x: 1 }] }, { a: [1, 2, { x: 1 }] }) &&
        !isDeepStrictEqual({ a: 1 }, { a: 2 }) &&
        isDeepStrictEqual(new Map([["k", 1]]), new Map([["k", 1]])) &&
        isDeepStrictEqual(new Set([1, 2]), new Set([1, 2])) &&
        !isDeepStrictEqual([1, 2], [1, 2, 3]);
    results.push(["deep-strict-equal", deepOk]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
