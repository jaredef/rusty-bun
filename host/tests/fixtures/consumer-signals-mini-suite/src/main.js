// Tier-J consumer #43: vendored signals-mini reactive library.
//
// Exercises the reactive-execution-context axis — global mutable
// observer pointer, dependency tracking via getter-side-effect,
// diamond resolution, batched updates. Different from all prior
// axes (parsing, async-control).

import signalsDefault, { signal, computed, effect, batch, untracked } from "signals-mini";

async function selfTest() {
    const results = [];

    // 1. Basic signal read/write.
    {
        const s = signal(1);
        s.value = 42;
        results.push(["signal-basic", s.value === 42]);
    }

    // 2. Effect tracks signal and re-runs.
    {
        const s = signal(1);
        const log = [];
        effect(() => log.push(s.value));
        s.value = 2;
        s.value = 3;
        results.push(["effect-tracks", JSON.stringify(log) === "[1,2,3]"]);
    }

    // 3. Computed value is lazy + cached.
    {
        const s = signal(2);
        let computes = 0;
        const sq = computed(() => { computes++; return s.value * s.value; });
        // Pre-read: no computes
        const a = sq.value;  // 4, 1 compute
        const b = sq.value;  // 4, cached
        s.value = 5;
        // Not read yet → recompute deferred
        const c = sq.value;  // 25, 2 computes total
        results.push(["computed-lazy-cached",
            a === 4 && b === 4 && c === 25 && computes === 2]);
    }

    // 4. Setting signal to same value (Object.is) doesn't fire effect.
    {
        const s = signal(1);
        let count = 0;
        effect(() => { s.value; count++; });
        s.value = 1;  // same value
        s.value = 1;  // same value
        results.push(["same-value-no-rerun", count === 1]);
    }

    // 5. Effect unsubscribes via dispose.
    {
        const s = signal(0);
        let count = 0;
        const dispose = effect(() => { s.value; count++; });
        s.value = 1;
        s.value = 2;
        dispose();
        s.value = 3;
        s.value = 4;
        results.push(["effect-dispose", count === 3]);  // initial + 2 firings
    }

    // 6. Diamond dependency: a → b, a → c, b+c → d. Updating a fires d once.
    {
        const a = signal(1);
        const b = computed(() => a.value + 1);
        const c = computed(() => a.value * 2);
        let dRuns = 0;
        effect(() => { b.value; c.value; dRuns++; });
        a.value = 10;
        // Initial run + one firing for a's change.
        results.push(["diamond-fires-once", dRuns === 2]);
    }

    // 7. Batch defers effect re-runs until end.
    {
        const a = signal(1);
        const b = signal(2);
        let runs = 0;
        effect(() => { a.value; b.value; runs++; });
        batch(() => {
            a.value = 10;
            b.value = 20;
        });
        // Initial run + one batched firing.
        results.push(["batch-coalesces", runs === 2]);
    }

    // 8. Nested batch only flushes at outermost end.
    {
        const a = signal(1);
        let runs = 0;
        effect(() => { a.value; runs++; });
        batch(() => {
            a.value = 2;
            batch(() => { a.value = 3; });
            a.value = 4;
        });
        results.push(["nested-batch", runs === 2]);  // initial + outer-end
    }

    // 9. peek() reads without subscribing.
    {
        const s = signal(1);
        let runs = 0;
        effect(() => { s.peek(); runs++; });
        s.value = 2;  // should NOT fire effect
        s.value = 3;
        results.push(["peek-no-subscribe", runs === 1]);
    }

    // 10. untracked() reads without subscribing.
    {
        const a = signal(1);
        const b = signal(10);
        let runs = 0;
        effect(() => {
            a.value;             // tracked
            untracked(() => b.value);  // not tracked
            runs++;
        });
        b.value = 20;  // shouldn't fire
        a.value = 2;   // should fire
        results.push(["untracked-no-subscribe", runs === 2]);
    }

    // 11. computed accessing computed (nested derivation).
    {
        const a = signal(2);
        const b = computed(() => a.value + 1);    // 3
        const c = computed(() => b.value * 10);   // 30
        results.push(["nested-computed-initial", c.value === 30]);
        a.value = 5;
        results.push(["nested-computed-propagates", c.value === 60]);
    }

    // 12. Effect re-subscribes correctly after dep changes.
    {
        const s1 = signal("a");
        const s2 = signal("b");
        const which = signal(true);
        let lastSeen = null;
        effect(() => { lastSeen = which.value ? s1.value : s2.value; });
        results.push(["initial-branch", lastSeen === "a"]);
        which.value = false;
        results.push(["switched-branch", lastSeen === "b"]);
        // Now s1 changes — should NOT fire (no longer a dep).
        s1.value = "a2";
        results.push(["unsubscribed-branch", lastSeen === "b"]);
        // s2 changes — should fire.
        s2.value = "b2";
        results.push(["new-branch-tracks", lastSeen === "b2"]);
    }

    // 13. Default-export shape.
    results.push(["default-export",
        signalsDefault.signal === signal &&
        signalsDefault.computed === computed &&
        signalsDefault.effect === effect &&
        signalsDefault.batch === batch &&
        signalsDefault.untracked === untracked]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
