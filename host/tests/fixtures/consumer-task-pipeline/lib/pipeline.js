// User-defined async iterable via Symbol.asyncIterator.
// Real consumer-class pattern: iterate while doing async work per step.

export class Pipeline {
    constructor(items) {
        this._items = items;
        this._observers = [];  // hooks called per yielded item
    }
    onItem(fn) { this._observers.push(fn); return this; }
    async *[Symbol.asyncIterator]() {
        for (const item of this._items) {
            // Simulate async work — Promise.resolve tick.
            await Promise.resolve();
            for (const fn of this._observers) fn(item);
            yield item;
        }
    }
}

// Generator delegation via yield* — compose two pipelines.
export async function* concatPipelines(a, b) {
    yield* a;
    yield* b;
}
