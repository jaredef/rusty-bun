// Tier-J consumer #57: node:events EventEmitter. Tier-Π3.8 closure round.
//
// EventEmitter is the most-imported npm builtin. The fixture exercises
// the patterns real packages depend on: subclassing for stream/server,
// once for promise-bridging, error-event-throws-without-handler, the
// removeListener semantics for one-shot cleanup (node-fetch), and the
// once(emitter, event) module-level helper.

import EventEmitter, { once as eventsOnce } from "node:events";
import { EventEmitter as EE2 } from "node:events";

async function selfTest() {
    const results = [];

    // 1. default export and named export are the same class.
    results.push(["default-eq-named", EventEmitter === EE2]);

    // 2. Basic on/emit.
    const e2 = new EventEmitter();
    let r2 = null;
    e2.on("ping", (msg) => { r2 = msg; });
    e2.emit("ping", "pong");
    results.push(["basic-on-emit", r2 === "pong"]);

    // 3. once fires only once.
    const e3 = new EventEmitter();
    let c3 = 0;
    e3.once("hit", () => { c3 += 1; });
    e3.emit("hit");
    e3.emit("hit");
    results.push(["once-fires-once", c3 === 1]);

    // 4. removeListener (and off) cleans up one-shot cleanup pattern.
    const e4 = new EventEmitter();
    const fn4 = () => {};
    e4.on("x", fn4);
    e4.off("x", fn4);
    results.push(["off-cleans", e4.listenerCount("x") === 0]);

    // 5. error event with no listener throws.
    const e5 = new EventEmitter();
    let threw = false;
    try { e5.emit("error", new Error("boom")); } catch (err) { threw = err.message === "boom"; }
    results.push(["error-throws-without-handler", threw]);

    // 6. error event with listener does NOT throw.
    const e6 = new EventEmitter();
    let captured = null;
    e6.on("error", (err) => { captured = err.message; });
    e6.emit("error", new Error("caught"));
    results.push(["error-with-handler-no-throw", captured === "caught"]);

    // 7. once(emitter, event) returns Promise<args[]>.
    const e7 = new EventEmitter();
    setTimeout(() => e7.emit("ready", "alpha", 42), 0);
    const r7 = await eventsOnce(e7, "ready");
    results.push(["events-once-promise",
        Array.isArray(r7) && r7[0] === "alpha" && r7[1] === 42]);

    // 8. Subclass via extends — the canonical npm pattern.
    class Server extends EventEmitter {
        listen() { this.emit("listening", 8080); }
    }
    const srv = new Server();
    let port = null;
    srv.on("listening", (p) => { port = p; });
    srv.listen();
    results.push(["subclass-emits", port === 8080]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
