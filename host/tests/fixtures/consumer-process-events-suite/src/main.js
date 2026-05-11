// Tier-J consumer #56: process EventEmitter + signal handler stubs +
// stdin stub. Tier-Π2.7 closure round.
//
// Strategy: exercise the EventEmitter shape the way real npm packages
// do (Express signal-handlers, graceful-shutdown helpers, koa-onerror).
// Signal-delivery is stubbed in rusty-bun-host (no real OS signals reach
// JS); handlers register and stay quiet, which matches consumer
// expectation that `process.on('SIGINT', cb)` doesn't throw.

async function selfTest() {
    const results = [];

    // 1. process.on returns process (chainable per Node convention).
    const r1 = process.on("custom-1", () => {});
    results.push(["process-on-returns-process", r1 === process]);

    // 2. process.emit fires registered listeners with arguments.
    let received = null;
    process.on("custom-2", (a, b) => { received = [a, b]; });
    process.emit("custom-2", "alpha", 42);
    results.push(["emit-fires-listener",
        Array.isArray(received) && received[0] === "alpha" && received[1] === 42]);

    // 3. listenerCount reports correct count.
    process.on("custom-3", () => {});
    process.on("custom-3", () => {});
    results.push(["listener-count", process.listenerCount("custom-3") === 2]);

    // 4. process.off removes a listener.
    const fn4 = () => {};
    process.on("custom-4", fn4);
    process.off("custom-4", fn4);
    results.push(["off-removes-listener", process.listenerCount("custom-4") === 0]);

    // 5. once fires only on first emit.
    let onceCount = 0;
    process.once("custom-5", () => { onceCount += 1; });
    process.emit("custom-5");
    process.emit("custom-5");
    results.push(["once-fires-once", onceCount === 1]);

    // 6. SIGINT handler registers without throwing (signal stubbed
    //    in rusty-bun-host; consumer code can register safely).
    let sigintOk = true;
    try {
        process.on("SIGINT", () => {});
        process.on("SIGTERM", () => {});
    } catch (e) {
        sigintOk = false;
    }
    results.push(["signal-handlers-register", sigintOk]);

    // 7. emit returns a truthy value (Node-compat: true). Bun matches.
    process.on("custom-7", () => {});
    results.push(["emit-return-value", process.emit("custom-7") === true]);

    // 8. process.stdin is present with the readable-stream API shape.
    //    isTTY is undefined when not on a TTY (Node + Bun convention).
    results.push(["stdin-shape",
        process.stdin != null &&
        typeof process.stdin.on === "function" &&
        typeof process.stdin.read === "function" &&
        typeof process.stdin.resume === "function" &&
        typeof process.stdin.pause === "function" &&
        !process.stdin.isTTY]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
