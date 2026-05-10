// Sync generator yielding sequence ids. Demonstrates function*, .return(),
// Array.from over a generator, spread of generator results.

export function* sequenceFrom(start, count) {
    let i = start;
    for (let n = 0; n < count; n++) {
        yield i;
        i++;
    }
}

// Generator delegation across two ranges.
export function* compound(startA, countA, startB, countB) {
    yield* sequenceFrom(startA, countA);
    yield* sequenceFrom(startB, countB);
}

// SequenceId class with Symbol.toPrimitive + Object.defineProperty accessor.
// Real consumer-class pattern: id objects that coerce to their numeric form
// in string contexts and template literals.
export class SequenceId {
    constructor(n) {
        this._n = n;
        Object.defineProperty(this, "n", {
            get() { return this._n; },
            enumerable: true,
            configurable: false,
        });
    }
    [Symbol.toPrimitive](hint) {
        // Per ECMAScript spec, arithmetic + on non-Date uses hint "default";
        // String() / template-literal coercion uses hint "string".
        if (hint === "string") return "id-" + this._n;
        return this._n;  // "number" and "default" both → numeric
    }
}
