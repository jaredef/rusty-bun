// Tier-J consumer #6: batch data-loader (spec-first M9 authoring).
//
// Exercises shapes not yet exercised by the prior six fixtures:
//   - Promise.all / Promise.allSettled / Promise.race (parallel + tolerant
//     + timeout patterns)
//   - Proxy with custom get trap + Reflect.get for default handling
//   - BigInt arithmetic + BigInt-keyed Map storage
//   - Tagged template literals
//   - Object.fromEntries from a parallel-fetch result
//   - Spread of Promise.all results into an object aggregator

import {
    fetchUser, fetchProfile, fetchSettings, fetchOptional, slowFetch,
} from "../lib/sources.js";
import { makeCache } from "../lib/cache.js";
import { q } from "../lib/tag.js";

async function selfTest() {
    const results = [];

    // 1. Promise.all: parallel fetch of three sources, aggregated.
    const userId = 7n;  // BigInt id
    const [user, profile, settings] = await Promise.all([
        fetchUser(userId),
        fetchProfile(userId),
        fetchSettings(userId),
    ]);
    results.push(["promise-all",
        user.name === "user-7" && profile.bio === "bio for 7" && settings.theme === "light"]);

    // 2. Promise.allSettled: tolerate partial failure.
    const ids = [1n, 2n, 3n, 4n];  // 3n fails per fetchOptional
    const settled = await Promise.allSettled(ids.map((id) => fetchOptional(id)));
    const fulfilled = settled.filter((s) => s.status === "fulfilled").length;
    const rejected = settled.filter((s) => s.status === "rejected").length;
    results.push(["promise-all-settled", fulfilled === 3 && rejected === 1]);

    // 3. Promise.race: timeout pattern — fast resolver wins.
    const winner = await Promise.race([
        slowFetch(99n, 100),  // would take 100ms
        Promise.resolve({ id: 99n, fast: true }),
    ]);
    results.push(["promise-race", winner.fast === true]);

    // 4. Proxy with BigInt-keyed Map storage.
    const cache = makeCache();
    cache.set(42n, { name: "answer" });
    cache.set(7n, { name: "seven" });
    results.push(["proxy-bigint-cache",
        cache["42"].name === "answer" && cache["7"].name === "seven" && cache.size === 2]);

    // 5. Reflect.has and Reflect.ownKeys for introspection.
    const inner = { a: 1, b: 2, [Symbol("hidden")]: "x" };
    const ownKeys = Reflect.ownKeys(inner);
    results.push(["reflect-introspect",
        Reflect.has(inner, "a") && Reflect.has(inner, "b") &&
        !Reflect.has(inner, "c") &&
        // ownKeys includes Symbol keys; length should be 3 (a, b, hidden).
        ownKeys.length === 3]);

    // 6. BigInt arithmetic — used as cache key offsets.
    const base = 1000n;
    const offsets = [1n, 2n, 3n];
    const keys = offsets.map((o) => base * 10n + o);
    results.push(["bigint-arithmetic",
        keys[0] === 10001n && keys[1] === 10002n && keys[2] === 10003n &&
        typeof keys[0] === "bigint"]);

    // 7. Tagged template literal — interpolation with BigInt/string handling.
    const queryStr = q`SELECT * FROM users WHERE id = ${userId} AND name = ${"Alice"} LIMIT ${10n}`;
    results.push(["tagged-template",
        queryStr === 'SELECT * FROM users WHERE id = 7 AND name = "Alice" LIMIT 10']);

    // 8. Object.fromEntries from a Promise.all result.
    const entries = await Promise.all([
        Promise.resolve(["a", 1]),
        Promise.resolve(["b", 2]),
        Promise.resolve(["c", 3]),
    ]);
    const obj = Object.fromEntries(entries);
    results.push(["object-fromentries", obj.a === 1 && obj.b === 2 && obj.c === 3]);

    // 9. Spread + Promise.all aggregation pattern.
    const aggregated = {
        ...(await fetchUser(99n)),
        ...(await fetchProfile(99n)),
        ...(await fetchSettings(99n)),
    };
    results.push(["spread-aggregate",
        aggregated.name === "user-99" && aggregated.bio === "bio for 99" &&
        aggregated.theme === "light"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");

process.stdout.write(summary + "\n");
