// Tier-J consumer #42: vendored async-pool library.
//
// Exercises the async/AbortSignal axis — different from the prior
// four parsing/rendering-axis vendored libraries. Tests AbortController,
// AbortSignal, Promise lifecycle, concurrency control, async ordering.

import asyncPoolDefault, { Pool, pool, AbortError } from "async-pool";

const sleep = (ms) => new Promise(r => setTimeout(r, ms));

async function selfTest() {
    const results = [];

    // 1. Basic concurrency: 5 tasks through pool of 2 — at most 2
    //    concurrent observed.
    {
        const p = new Pool({ concurrency: 2 });
        let active = 0;
        let maxActive = 0;
        const tasks = [];
        for (let i = 0; i < 5; i++) {
            tasks.push(p.run(async () => {
                active++;
                maxActive = Math.max(maxActive, active);
                await sleep(10);
                active--;
                return i;
            }));
        }
        const out = await Promise.all(tasks);
        results.push(["concurrency-bound",
            maxActive === 2 && out.length === 5 && out.every((v, i) => v === i)]);
    }

    // 2. onIdle resolves after all tasks complete.
    {
        const p = new Pool({ concurrency: 3 });
        for (let i = 0; i < 4; i++) p.run(async () => { await sleep(5); });
        await p.onIdle();
        results.push(["on-idle-resolves",
            p.active === 0 && p.size === 0]);
    }

    // 3. onIdle on an already-empty pool resolves immediately.
    {
        const p = new Pool({ concurrency: 2 });
        const start = Date.now();
        await p.onIdle();
        const elapsed = Date.now() - start;
        results.push(["on-idle-empty-immediate", elapsed < 5]);
    }

    // 4. AbortSignal pre-aborted at enqueue → rejects without running.
    {
        const p = new Pool({ concurrency: 2 });
        const ctrl = new AbortController();
        ctrl.abort("already cancelled");
        let ran = false;
        let caught = null;
        try {
            await p.run(async () => { ran = true; }, { signal: ctrl.signal });
        } catch (e) { caught = e; }
        results.push(["pre-aborted-rejects-without-run",
            ran === false && caught instanceof AbortError]);
    }

    // 5. AbortSignal aborts while task is queued → task is skipped.
    {
        const p = new Pool({ concurrency: 1 });
        const blocker = p.run(async () => { await sleep(20); });  // blocks slot
        const ctrl = new AbortController();
        let ran = false;
        const promise = p.run(async () => { ran = true; }, { signal: ctrl.signal });
        ctrl.abort();
        let caught = null;
        try { await promise; } catch (e) { caught = e; }
        await blocker;
        results.push(["queued-task-aborted",
            ran === false && caught instanceof AbortError]);
    }

    // 6. AbortSignal aborts mid-flight → task's signal observes abort.
    {
        const p = new Pool({ concurrency: 1 });
        const ctrl = new AbortController();
        let observedAbort = false;
        const promise = p.run(async ({ signal }) => {
            signal.addEventListener("abort", () => { observedAbort = true; });
            await sleep(20);
            return "completed";
        }, { signal: ctrl.signal });
        await sleep(5);
        ctrl.abort();
        let caught = null;
        try { await promise; } catch (e) { caught = e; }
        results.push(["mid-flight-abort-observed",
            observedAbort === true && caught instanceof AbortError]);
    }

    // 7. pool() helper: run a batch with concurrency.
    {
        const tasks = Array.from({ length: 6 }, (_, i) => async () => {
            await sleep(2);
            return i * 10;
        });
        const out = await pool(3, tasks);
        results.push(["pool-helper",
            out.length === 6 && out[5] === 50]);
    }

    // 8. abortAll cancels all queued AND in-flight.
    {
        const p = new Pool({ concurrency: 2 });
        const promises = [];
        for (let i = 0; i < 5; i++) {
            promises.push(p.run(async () => { await sleep(50); return i; }));
        }
        await sleep(5);
        p.abortAll("test cancel");
        const settled = await Promise.allSettled(promises);
        const rejected = settled.filter(s => s.status === "rejected").length;
        results.push(["abort-all", rejected === 5]);
    }

    // 9. Task throws → rejects without contaminating other tasks.
    {
        const p = new Pool({ concurrency: 2 });
        const ok = p.run(async () => "ok");
        const bad = p.run(async () => { throw new Error("boom"); });
        const okOut = await ok;
        let caught = null;
        try { await bad; } catch (e) { caught = e; }
        results.push(["task-error-isolated",
            okOut === "ok" && caught && caught.message === "boom"]);
    }

    // 10. Tasks run in FIFO order at concurrency=1.
    {
        const p = new Pool({ concurrency: 1 });
        const order = [];
        const promises = [];
        for (let i = 0; i < 5; i++) {
            promises.push(p.run(async () => { order.push(i); }));
        }
        await Promise.all(promises);
        results.push(["fifo-at-concurrency-1",
            JSON.stringify(order) === "[0,1,2,3,4]"]);
    }

    // 11. Default export shape.
    results.push(["default-export",
        asyncPoolDefault.Pool === Pool &&
        asyncPoolDefault.pool === pool &&
        asyncPoolDefault.AbortError === AbortError]);

    // 12. Invalid concurrency throws TypeError.
    {
        let caught = null;
        try { new Pool({ concurrency: 0 }); } catch (e) { caught = e; }
        results.push(["invalid-concurrency",
            caught instanceof TypeError]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
