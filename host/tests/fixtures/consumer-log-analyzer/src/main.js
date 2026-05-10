// Tier-J consumer #7: log analyzer (spec-first M9 authoring).
//
// Picked axes that test the basin's coverage without invoking the
// known-out-of-basin features (WeakRef per bug-catcher E7).
// Exercises shapes not yet exercised:
//   - Regex with named capture groups (?<name>...) — pattern named extraction
//   - Regex global flag with .matchAll() iteration
//   - Date arithmetic via .getTime() differences and ISO parse
//   - Array.reduce with seed-as-Map (in-place accumulator)
//   - Array.sort with comparator function returning negative/zero/positive
//   - Map insertion-order preservation and iteration
//   - Object.entries → sort → Object.fromEntries pipeline
//   - String.prototype.padStart for fixed-width output

import { parseLine, extractDurations } from "../lib/parser.js";
import { groupByLevel, sortByTimestamp, durationStats, summarizeByLevel } from "../lib/stats.js";

const RAW_LINES = [
    "2024-01-15T10:23:45.000Z [INFO] http.server: request took 42ms",
    "2024-01-15T10:23:46.000Z [WARN] http.server: slow query took 350ms",
    "2024-01-15T10:23:44.000Z [INFO] db.pool: connection acquired",
    "2024-01-15T10:23:47.000Z [ERROR] http.server: handler took 1200ms and failed",
    "2024-01-15T10:23:48.000Z [INFO] http.server: request took 18ms",
    "2024-01-15T10:23:49.000Z [DEBUG] cache: hit ratio 0.92",
    "malformed line that won't parse",
    "2024-01-15T10:23:50.000Z [ERROR] auth: login failed for user x",
];

async function selfTest() {
    const results = [];

    // 1. Named-capture regex parse.
    const parsed = RAW_LINES.map(parseLine);
    const valid = parsed.filter(Boolean);
    results.push(["regex-named-capture",
        valid.length === 7 &&
        valid[0].level === "INFO" &&
        valid[0].component === "http.server" &&
        valid[0].timestamp.toISOString() === "2024-01-15T10:23:45.000Z"]);

    // 2. matchAll over a global regex with named groups.
    const durs = extractDurations("first took 42ms; later took 350ms; also took 1200ms here");
    results.push(["regex-matchAll", JSON.stringify(durs) === "[42,350,1200]"]);

    // 3. Sort by timestamp — verifies stable .getTime() ordering.
    const sorted = sortByTimestamp(valid);
    const firstTs = sorted[0].timestamp.toISOString();
    const lastTs = sorted[sorted.length - 1].timestamp.toISOString();
    results.push(["sort-by-time",
        firstTs === "2024-01-15T10:23:44.000Z" &&
        lastTs === "2024-01-15T10:23:50.000Z"]);

    // 4. Date arithmetic — total time span in seconds.
    const span = (sorted[sorted.length - 1].timestamp.getTime() - sorted[0].timestamp.getTime()) / 1000;
    results.push(["date-arithmetic", span === 6]);

    // 5. Group-by via Array.reduce with Map seed.
    const grouped = groupByLevel(valid);
    results.push(["group-by-reduce",
        grouped.get("INFO").length === 3 &&
        grouped.get("WARN").length === 1 &&
        grouped.get("ERROR").length === 2 &&
        grouped.get("DEBUG").length === 1]);

    // 6. Object.entries → sort → Object.fromEntries pipeline with
    // severity ordering.
    const summary = summarizeByLevel(grouped);
    const orderedKeys = Object.keys(summary);
    results.push(["object-entries-pipeline",
        JSON.stringify(orderedKeys) === '["DEBUG","INFO","WARN","ERROR"]']);

    // 7. Aggregate durations across all parsed messages.
    const allDurations = valid.flatMap((e) => extractDurations(e.message));
    const stats = durationStats(allDurations);
    results.push(["duration-stats",
        stats.count === 4 &&
        stats.min === 18 &&
        stats.max === 1200 &&
        stats.sum === 1610 &&
        stats.mean === 402.5]);

    // 8. padStart for fixed-width table-like output (no Intl).
    const tableLine = String(stats.mean).padStart(8, " ") + " ms";
    results.push(["padStart", tableLine === "   402.5 ms"]);

    // 9. Map iteration order preservation — keys returned in insertion order.
    // groupByLevel inserts in encounter order; valid[0] is INFO so INFO is first.
    const groupedKeys = [...grouped.keys()];
    results.push(["map-iteration-order",
        JSON.stringify(groupedKeys) === '["INFO","WARN","ERROR","DEBUG"]']);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

if (typeof process !== "undefined" && process.stdout && process.stdout.write) {
    process.stdout.write(summary + "\n");
} else {
    globalThis.__esmResult = summary;
}
