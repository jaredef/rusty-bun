// Tier-J consumer #20: ES2025 Set algebra (M9.bis-native).
//
// Exercises the Set.prototype methods that close E.10 in this round.
// Bun has them natively; rusty-bun-host gets them via polyfill installed
// in install_set_methods_polyfill.

async function selfTest() {
    const results = [];

    const a = new Set([1, 2, 3, 4]);
    const b = new Set([3, 4, 5, 6]);
    const c = new Set([7, 8]);

    // 1. union: A ∪ B
    const u = a.union(b);
    results.push(["union",
        u instanceof Set && u.size === 6 &&
        [1, 2, 3, 4, 5, 6].every((v) => u.has(v))]);

    // 2. intersection: A ∩ B
    const i = a.intersection(b);
    results.push(["intersection",
        i instanceof Set && i.size === 2 &&
        i.has(3) && i.has(4)]);

    // 3. difference: A \ B
    const d = a.difference(b);
    results.push(["difference",
        d instanceof Set && d.size === 2 &&
        d.has(1) && d.has(2)]);

    // 4. symmetricDifference: A △ B
    const sd = a.symmetricDifference(b);
    results.push(["symmetricDifference",
        sd instanceof Set && sd.size === 4 &&
        sd.has(1) && sd.has(2) && sd.has(5) && sd.has(6)]);

    // 5. isSubsetOf
    results.push(["isSubsetOf",
        new Set([1, 2]).isSubsetOf(a) === true &&
        new Set([1, 99]).isSubsetOf(a) === false]);

    // 6. isSupersetOf
    results.push(["isSupersetOf",
        a.isSupersetOf(new Set([1, 2])) === true &&
        a.isSupersetOf(new Set([1, 99])) === false]);

    // 7. isDisjointFrom
    results.push(["isDisjointFrom",
        a.isDisjointFrom(c) === true &&
        a.isDisjointFrom(b) === false]);

    // 8. Originals not mutated.
    results.push(["originals-immutable",
        a.size === 4 && b.size === 4]);

    // 9. Accept iterables, not just Sets (per spec — at least the
    // Set-or-iterable forms are interoperable).
    const fromArr = a.intersection(new Set([3, 4]));
    results.push(["set-input", fromArr.size === 2 && fromArr.has(3)]);

    // 10. Composition: build a deduplicated symbol-table.
    const reserved = new Set(["if", "else", "while", "return"]);
    const identifiers = new Set(["x", "y", "if", "z", "while"]);
    const userIds = identifiers.difference(reserved);
    results.push(["composition",
        userIds.size === 3 &&
        userIds.has("x") && userIds.has("y") && userIds.has("z") &&
        !userIds.has("if") && !userIds.has("while")]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
