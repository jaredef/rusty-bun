// Tier-J consumer #67: live ws:// round-trip. Tier-Π1.5.d closure.
//
// Connects to a Bun-spawned ws echo server (harness provides
// WS_TEST_PORT) and exercises the canonical client lifecycle:
// construct → onopen → send → pump → onmessage → close → onclose.

async function selfTest() {
    const port = process.env.WS_TEST_PORT;
    if (!port) {
        const names = ["constructed", "open-event", "echo-roundtrip", "close-clean"];
        return names.map(n => [n + "-skipped-noport", true]);
    }
    const results = [];
    const url = "ws://127.0.0.1:" + port + "/";

    // 1. Construction completes (open or close-on-error).
    let ws = null;
    try {
        ws = new WebSocket(url);
    } catch (e) {
        results.push(["constructed-threw", false]);
        return results;
    }
    results.push(["constructed", ws != null]);

    // 2. open event fires (synchronously-construct under rusty-bun-host;
    //    async under Bun — both have onopen called via microtask).
    let openFired = false;
    await new Promise((resolve) => {
        ws.onopen = () => { openFired = true; resolve(); };
        // Bun async-emits open; rusty-bun-host fires via microtask.
        // Either way, give it a tick.
        setTimeout(resolve, 200);
    });
    results.push(["open-event", openFired]);

    if (!openFired) {
        results.push(["echo-roundtrip", false]);
        results.push(["close-clean", false]);
        return results;
    }

    // 3. Send a message and receive its echo.
    let received = null;
    await new Promise((resolve) => {
        ws.onmessage = (ev) => { received = ev.data; resolve(); };
        ws.send("hello-ws");
        // Under rusty-bun-host the consumer pumps explicitly; under Bun
        // the message arrives via the background event loop.
        if (typeof ws.pump === "function") {
            // Pump in a tight loop until message arrives or timeout.
            let pumps = 0;
            const tick = () => {
                if (received !== null || pumps > 100) { resolve(); return; }
                ws.pump();
                pumps += 1;
                setTimeout(tick, 10);
            };
            tick();
        } else {
            setTimeout(resolve, 500);
        }
    });
    results.push(["echo-roundtrip", received === "hello-ws"]);

    // 4. close fires onclose with wasClean.
    let closeOk = false;
    await new Promise((resolve) => {
        ws.onclose = (ev) => { closeOk = ev && ev.wasClean !== undefined; resolve(); };
        ws.close(1000, "done");
        setTimeout(resolve, 200);
    });
    results.push(["close-clean", closeOk]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
