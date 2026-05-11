// Tier-J consumer #9: sequence-id generator (spec-first M9 authoring).
//
// In-basin axes per probes 2026-05-10:
//   - Atomics (object, in-basin) — Atomics.add, .load, .compareExchange
//   - SharedArrayBuffer (function, in-basin)
//   - Sync generators (function*) — confirmed by prior async-gen fixtures
//   - Array.from with iterable argument
//   - Spread of generator results into array literal
//   - String.raw tagged template
//   - Object.defineProperty with accessor descriptor
//   - Symbol.toPrimitive on user-defined class
//
// Stays clear of E.7 (WeakRef), E.8 (subtle.importKey), E.9 (Intl/WS/etc).

import { makeAtomicsCounter } from "../lib/counter.js";
import { sequenceFrom, compound, SequenceId } from "../lib/sequence.js";

async function selfTest() {
    const results = [];

    // 1. Atomics-backed counter increments lock-free.
    const c = makeAtomicsCounter();
    const ids = [c.next(), c.next(), c.next()];
    results.push(["atomics-counter",
        ids[0] === 1 && ids[1] === 2 && ids[2] === 3 && c.value === 3]);

    // 2. Atomics.compareExchange — CAS reset succeeds when expected matches.
    const reset1 = c.casReset(3);
    const reset2 = c.casReset(99);  // expected wrong; CAS fails
    results.push(["atomics-cas",
        reset1 === true && c.value === 0 && reset2 === false]);

    // 3. Sync generator with Array.from.
    const gen = sequenceFrom(10, 5);
    const arr = Array.from(gen);
    results.push(["array-from-generator", arr.join(",") === "10,11,12,13,14"]);

    // 4. Spread of generator into array literal.
    const spread = [...sequenceFrom(100, 3)];
    results.push(["spread-generator", spread.join(",") === "100,101,102"]);

    // 5. Generator delegation via yield* across two ranges.
    const both = [...compound(1, 3, 10, 2)];
    results.push(["yield-delegation", both.join(",") === "1,2,3,10,11"]);

    // 6. Generator .return() short-circuits cleanly.
    const g = sequenceFrom(0, 100);
    const taken = [];
    for (const v of g) {
        taken.push(v);
        if (taken.length >= 3) {
            g.return();  // stops the generator
            break;
        }
    }
    // After .return(), further .next() reports done.
    const after = g.next();
    results.push(["generator-return",
        taken.join(",") === "0,1,2" && after.done === true]);

    // 7. String.raw tagged template — escapes preserved verbatim.
    const raw = String.raw`line1\nline2\t${"tabbed"}`;
    results.push(["string-raw",
        raw === "line1\\nline2\\ttabbed"]);

    // 8. Object.defineProperty with accessor + Symbol.toPrimitive coercion.
    const id = new SequenceId(42);
    // Numeric coercion in arithmetic uses Symbol.toPrimitive("number").
    const sum = id + 8;
    // String coercion in template literal uses Symbol.toPrimitive("string").
    const tmpl = `the value is ${id}`;
    // Direct property access via the defineProperty accessor.
    results.push(["defineProperty-toPrimitive",
        sum === 50 && tmpl === "the value is id-42" && id.n === 42]);

    // 9. Object.defineProperty enforces non-configurable.
    let defineFailed = false;
    try {
        Object.defineProperty(id, "n", { value: 999, writable: true });
    } catch (e) {
        defineFailed = e instanceof TypeError;
    }
    results.push(["defineProperty-nonconfigurable", defineFailed]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
