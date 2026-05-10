// Aggregation helpers exercising Array.reduce, sort with comparator,
// Map insertion-order preservation, Object.fromEntries.

export function groupByLevel(entries) {
    return entries.reduce((acc, e) => {
        const list = acc.get(e.level) || [];
        list.push(e);
        acc.set(e.level, list);
        return acc;
    }, new Map());
}

export function sortByTimestamp(entries) {
    return [...entries].sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
}

export function durationStats(values) {
    if (values.length === 0) return { count: 0, min: 0, max: 0, sum: 0, mean: 0 };
    const sorted = [...values].sort((a, b) => a - b);
    const sum = sorted.reduce((s, v) => s + v, 0);
    return {
        count: sorted.length,
        min: sorted[0],
        max: sorted[sorted.length - 1],
        sum,
        mean: sum / sorted.length,
    };
}

// Deterministic level severity for ordering.
const LEVEL_RANK = { TRACE: 0, DEBUG: 1, INFO: 2, WARN: 3, ERROR: 4, FATAL: 5 };

export function summarizeByLevel(grouped) {
    const out = {};
    for (const [level, list] of grouped) {
        out[level] = list.length;
    }
    // Object.entries → sort by level rank → Object.fromEntries.
    const ordered = Object.entries(out)
        .sort(([a], [b]) => (LEVEL_RANK[a] ?? 99) - (LEVEL_RANK[b] ?? 99));
    return Object.fromEntries(ordered);
}
