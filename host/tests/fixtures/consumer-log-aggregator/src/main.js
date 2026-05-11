// Tier-J consumer #4: log-aggregator (spec-first M9 authoring).
//
// Exercises shapes the prior three fixtures didn't:
//   - node:path module-import (basename / dirname / extname)
//   - User-defined event-emitter (Map-backed listener registry,
//     on/off/emit/listenerCount)
//   - structuredClone defensive copy on emit (Date + Map + nested
//     objects all cloned per payload)
//   - URLSearchParams filter-query construction
//   - JSON.stringify pretty-printing with stable key order
//   - Array.prototype.flatMap, .filter, .map cross-pilot composition

import { Emitter } from "../lib/emitter.js";
import { makeLogRecord, buildQuery } from "../lib/log-record.js";

const emitter = new Emitter();
const collected = [];
const errorsOnly = [];

emitter.on("log", (rec) => collected.push(rec));
emitter.on("log", (rec) => {
    if (rec.level === "error") errorsOnly.push(rec);
});

const sources = [
    ["/var/log/app/server.log", "info", "server started", ["startup", "core"]],
    ["/var/log/app/server.log", "warn", "slow query", ["db", "perf"]],
    ["/var/log/app/auth.log", "error", "login failed", ["auth", "security"]],
    ["/var/log/cron/daily.log", "info", "rotation complete", ["cron"]],
    ["/var/log/app/auth.log", "error", "session expired", ["auth"]],
];

for (const [src, level, msg, tags] of sources) {
    emitter.emit("log", makeLogRecord(src, level, msg, tags));
}

async function selfTest() {
    const results = [];

    results.push(["all-collected", collected.length === 5]);
    results.push(["errors-only", errorsOnly.length === 2]);
    results.push(["listener-count", emitter.listenerCount("log") === 2]);

    // node:path correctness across records.
    const dirs = collected.map((r) => r.source.dir).filter((d, i, a) => a.indexOf(d) === i).sort();
    results.push(["path-dirname", JSON.stringify(dirs) === '["/var/log/app","/var/log/cron"]']);

    const bases = collected.map((r) => r.source.base).filter((b, i, a) => a.indexOf(b) === i).sort();
    results.push(["path-basename", JSON.stringify(bases) === '["auth.log","daily.log","server.log"]']);

    // structuredClone defensive copy: mutating a collected record must
    // not affect the emitter's source.
    const c0 = collected[0];
    c0.tags.set("MUTATED", true);
    c0.when.setTime(99999);
    // Re-emit one and verify the new clone is independent of the prior
    // mutation.
    emitter.emit("log", makeLogRecord("/tmp/x.log", "info", "fresh", ["fresh"]));
    const c5 = collected[5];
    results.push(["clone-independence",
        !c5.tags.has("MUTATED") && c5.when.getTime() === 0]);

    // URLSearchParams filter building.
    const q1 = buildQuery({ level: "error" });
    const q2 = buildQuery({ tag: "auth", dir: "/var/log/app" });
    results.push(["query-build",
        q1 === "level=error" &&
        q2 === "tag=auth&dir=%2Fvar%2Flog%2Fapp"]);

    // Cross-pilot: flatMap of tag arrays from error records, dedup, sort.
    const errorTags = errorsOnly
        .flatMap((r) => Array.from(r.tags.keys()))
        .filter((t, i, a) => a.indexOf(t) === i)
        .sort();
    results.push(["flatmap-tags", JSON.stringify(errorTags) === '["auth","security"]']);

    // JSON pretty-print with stable shape. Serialize a single record's
    // primitive fields (Date/Map → string/array via custom handling).
    const summary = JSON.stringify({
        source: c5.source,
        level: c5.level,
        message: c5.message,
    }, null, 2);
    results.push(["json-pretty", summary.includes('"level": "info"') && summary.includes('"base": "x.log"')]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
