// Canonical-JSON: deterministic key ordering. Required for signing —
// non-canonical output would produce non-reproducible digests.
export function canonicalJson(value) {
    if (value === null || typeof value !== "object" || Array.isArray(value)) {
        return JSON.stringify(value);
    }
    const keys = Object.keys(value).sort();
    const parts = keys.map((k) => JSON.stringify(k) + ":" + canonicalJson(value[k]));
    return "{" + parts.join(",") + "}";
}
