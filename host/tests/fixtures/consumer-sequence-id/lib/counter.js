// Atomics-backed counter on a SharedArrayBuffer-backed Int32Array.
// Real consumer pattern: lock-free CAS even in single-threaded contexts.

export function makeAtomicsCounter() {
    const sab = new SharedArrayBuffer(4);
    const view = new Int32Array(sab);
    return {
        next() {
            // Atomics.add returns the previous value; new value = prev + 1.
            return Atomics.add(view, 0, 1) + 1;
        },
        get value() {
            return Atomics.load(view, 0);
        },
        // Compare-and-swap reset: only resets if currently equals expected.
        casReset(expected) {
            return Atomics.compareExchange(view, 0, expected, 0) === expected;
        },
    };
}
