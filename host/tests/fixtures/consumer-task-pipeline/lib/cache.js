// WeakMap-backed memoizer. Keys must be objects (WeakMap requirement).
// Real consumer-cache pattern: cache by reference identity, GC-friendly.

export function memoize(fn) {
    const cache = new WeakMap();
    return function memoized(key, ...rest) {
        if (cache.has(key)) return cache.get(key);
        const v = fn(key, ...rest);
        cache.set(key, v);
        return v;
    };
}
