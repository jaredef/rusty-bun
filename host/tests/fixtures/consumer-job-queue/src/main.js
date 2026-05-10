// Tier-J consumer #5: priority job queue with async-generator drain.
//
// Spec-first M9 authoring. Exercises shapes the prior fixtures didn't:
//   - Class inheritance (BaseJob → Job → PriorityJob), super(), instanceof
//   - Async generator (async function* + for-await-of consumer)
//   - node:crypto.randomUUID via ESM import (not the globalThis form)
//   - Custom Error subclasses (Error subclassing + .name + extra fields)
//   - Symbol-keyed private state + Symbol export across modules
//   - JSON.stringify with replacer function (filter symbol-keyed slots)
//   - Deterministic UUID v4 format check (no fixed values; format-only)

import { randomUUID } from "node:crypto";
import { BaseJob, Job, PriorityJob, kInternal } from "../lib/jobs.js";
import { PriorityQueue } from "../lib/queue.js";
import { InvalidJobError, QueueClosedError } from "../lib/errors.js";

async function selfTest() {
    const results = [];

    // 1. Class hierarchy correctness.
    const pj = new PriorityJob("send-email", { to: "x@y" }, 5);
    results.push(["inheritance",
        pj instanceof PriorityJob && pj instanceof Job && pj instanceof BaseJob]);

    // 2. node:crypto.randomUUID via ESM import yields valid v4.
    const id = randomUUID();
    results.push(["node-crypto-import",
        id.length === 36 && id[14] === "4" && /^[0-9a-f-]+$/.test(id)]);

    // 3. Custom error class subclassing.
    let caught = null;
    try { new Job(""); } catch (e) { caught = e; }
    results.push(["custom-error",
        caught instanceof InvalidJobError &&
        caught instanceof Error &&
        caught.name === "InvalidJobError"]);

    // 4. Priority ordering in the queue.
    const q = new PriorityQueue();
    q.enqueue(new PriorityJob("low", {}, 1));
    q.enqueue(new PriorityJob("high", {}, 9));
    q.enqueue(new PriorityJob("med", {}, 5));
    const order = [];
    while (q.size() > 0) order.push(q.dequeue().kind);
    results.push(["priority-order", order.join(",") === "high,med,low"]);

    // 5. Async generator drain via for-await-of.
    const q2 = new PriorityQueue();
    q2.enqueue(new PriorityJob("a", {}, 3));
    q2.enqueue(new PriorityJob("b", {}, 1));
    q2.enqueue(new PriorityJob("c", {}, 2));
    const drained = [];
    for await (const j of q2.drain()) {
        drained.push(j.kind);
    }
    results.push(["async-generator", drained.join(",") === "a,c,b"]);

    // 6. Queue-closed error propagation.
    const q3 = new PriorityQueue();
    q3.close();
    let closeCaught = null;
    try { q3.enqueue(new PriorityJob("x")); }
    catch (e) { closeCaught = e; }
    results.push(["queue-closed",
        closeCaught instanceof QueueClosedError && q3.closed() === true]);

    // 7. JSON.stringify with replacer — filter internal symbol-keyed state.
    // Symbol-keyed properties are not enumerable in JSON.stringify by
    // default, so even without a replacer they're excluded. The replacer
    // here additionally filters out one regular field for explicit control.
    const j = new PriorityJob("export-test", { x: 1 }, 7);
    j.markStarted();
    const replacer = (key, value) => {
        if (key === "createdAt") return undefined;  // omit
        return value;
    };
    const json = JSON.stringify(j, replacer);
    const parsed = JSON.parse(json);
    results.push(["json-replacer",
        parsed.kind === "export-test" &&
        parsed.priority === 7 &&
        parsed.payload.x === 1 &&
        !("createdAt" in parsed) &&
        !(kInternal.toString() in parsed)]);

    // 8. Internal state visibility through the kInternal Symbol export.
    // The Symbol is exported so consumers CAN reach the internal slot
    // if they import the symbol. This is a common encapsulation idiom.
    const internalState = j[kInternal];
    results.push(["symbol-export-access",
        internalState.startedAt === 1 && internalState.completedAt === null]);

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
