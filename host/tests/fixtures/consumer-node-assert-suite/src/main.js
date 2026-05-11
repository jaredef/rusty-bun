// Tier-J consumer #63: node:assert. Tier-Π3-extension closure round.
//
// node:assert is the canonical test-framework primitive most npm test
// infrastructure uses as a fallback (mocha, jest, ava, tap all import
// assert from node:assert in their core path). The fixture exercises
// the patterns real test code uses.

import assert, { AssertionError } from "node:assert";
import strict from "node:assert/strict";

async function selfTest() {
    const results = [];

    // 1. assert(value) passes on truthy, throws on falsy.
    let ok1 = true;
    try { assert(1); assert("x"); assert([]); } catch (_) { ok1 = false; }
    let threw1 = false;
    try { assert(0); } catch (e) { threw1 = e instanceof AssertionError; }
    results.push(["assert-callable", ok1 === true && threw1 === true]);

    // 2. assert.strictEqual on Object.is.
    let ok2 = true;
    try { assert.strictEqual(1, 1); } catch (_) { ok2 = false; }
    let threw2 = false;
    try { assert.strictEqual(1, "1"); } catch (e) { threw2 = e instanceof AssertionError; }
    results.push(["strict-equal", ok2 === true && threw2 === true]);

    // 3. assert.deepStrictEqual on nested objects.
    let ok3 = true;
    try { assert.deepStrictEqual({ a: [1, { b: 2 }] }, { a: [1, { b: 2 }] }); } catch (_) { ok3 = false; }
    let threw3 = false;
    try { assert.deepStrictEqual({ a: 1 }, { a: 2 }); } catch (e) { threw3 = e instanceof AssertionError; }
    results.push(["deep-strict-equal", ok3 === true && threw3 === true]);

    // 4. assert.throws with RegExp matcher.
    let ok4 = true;
    try {
        assert.throws(() => { throw new Error("boom"); }, /boom/);
    } catch (_) { ok4 = false; }
    let threw4 = false;
    try {
        assert.throws(() => { throw new Error("boom"); }, /nomatch/);
    } catch (e) { threw4 = e instanceof AssertionError; }
    results.push(["throws-regexp", ok4 === true && threw4 === true]);

    // 5. assert.doesNotThrow passes when fn doesn't throw.
    let ok5 = true;
    try { assert.doesNotThrow(() => 1 + 1); } catch (_) { ok5 = false; }
    results.push(["does-not-throw", ok5 === true]);

    // 6. assert.match for string-regexp.
    let ok6 = true;
    try { assert.match("hello world", /world/); } catch (_) { ok6 = false; }
    results.push(["match", ok6 === true]);

    // 7. assert.rejects with async function.
    let ok7 = true;
    try {
        await assert.rejects(async () => { throw new Error("async boom"); }, /async boom/);
    } catch (_) { ok7 = false; }
    results.push(["rejects-async", ok7 === true]);

    // 8. strict mode: strict.equal === strictEqual (not loose).
    let strictWorked = true;
    try { strict.equal(1, 1); } catch (_) { strictWorked = false; }
    let strictThrew = false;
    try { strict.equal(1, "1"); } catch (e) { strictThrew = e instanceof AssertionError; }
    results.push(["strict-mode", strictWorked === true && strictThrew === true]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
