// BigInt-keyed cache wrapped in a Proxy. The Proxy intercepts indexed
// access via numeric/BigInt keys and routes to the underlying Map.
// Real consumer code (DataLoader, etc.) uses similar wrappers.

export function makeCache() {
    const store = new Map();
    return new Proxy(store, {
        get(target, prop) {
            if (prop === "set" || prop === "has" || prop === "size" ||
                prop === "delete" || prop === Symbol.iterator) {
                const v = Reflect.get(target, prop);
                return typeof v === "function" ? v.bind(target) : v;
            }
            // Numeric-looking property lookups → Map.get with coerced key.
            // BigInt-shaped string properties (e.g. "42") route to Map.
            if (typeof prop === "string" && /^\d+$/.test(prop)) {
                return target.get(BigInt(prop));
            }
            return Reflect.get(target, prop);
        }
    });
}
