// Tier-J consumer #66: WHATWG WebSocket class API-shape.
// Tier-Π1.5.c structural validation. Live ws:// round-trip is the
// Π1.5.d round.
//
// Bun and rusty-bun-host both expose a global WebSocket class with
// the WHATWG interface. The fixture validates the structural shape
// without needing a live server: type checks on constants, properties,
// methods, and error semantics for invalid schemes / closed-state ops.

async function selfTest() {
    const results = [];

    // 1. Class is present as a global.
    results.push(["websocket-class-present",
        typeof globalThis.WebSocket === "function"]);

    // 2. Static readyState constants.
    results.push(["constants-shape",
        WebSocket.CONNECTING === 0 &&
        WebSocket.OPEN === 1 &&
        WebSocket.CLOSING === 2 &&
        WebSocket.CLOSED === 3]);

    // 3. Constructor accepts unrecognized scheme rejection. Both Bun
    //    and rusty-bun-host are lax on http:/https: (silently mapped
    //    to ws:/wss: per Bun convention); they should reject something
    //    truly invalid like "ftp://".
    let threwBadScheme = false;
    try { new WebSocket("ftp://example.com/"); }
    catch (e) { threwBadScheme = e instanceof SyntaxError || e instanceof TypeError; }
    results.push(["rejects-non-ws-scheme", threwBadScheme]);

    // 4. Constructor with ws:// to unreachable host transitions to CLOSED.
    //    (Async-emit close per the documented Tier-3 divergence.)
    let unreachableResult = null;
    await new Promise((resolve) => {
        try {
            const ws = new WebSocket("ws://nonexistent-host-99999.invalid:80/");
            // Synchronous-construct under rusty-bun-host: should already
            // be CLOSED with onclose scheduled. Bun async-constructs.
            if (ws.readyState === 3) {
                ws.onclose = (ev) => { unreachableResult = "closed:" + (ev.wasClean === false); resolve(); };
                // If onclose already fired via microtask, give it a tick.
                queueMicrotask(() => { if (!unreachableResult) { unreachableResult = "no-close-event"; resolve(); } });
            } else {
                ws.onclose = (ev) => { unreachableResult = "closed-async:" + (ev.wasClean === false); resolve(); };
                ws.onerror = () => {};
                // Bun: emits error+close after a delay.
                setTimeout(() => { if (!unreachableResult) { unreachableResult = "timeout"; resolve(); } }, 100);
            }
        } catch (e) {
            unreachableResult = "threw:" + e.message;
            resolve();
        }
    });
    // Both implementations should end up with the WebSocket in a closed
    // state with wasClean=false; the exact onclose timing differs but
    // the structural outcome is consistent.
    results.push(["unreachable-closes",
        unreachableResult !== null && /closed/.test(unreachableResult)]);

    // 5. binaryType getter/setter shape.
    let binaryTypeOk = false;
    try {
        // We can probe the getter/setter on the class prototype without
        // an actual connection.
        const desc = Object.getOwnPropertyDescriptor(WebSocket.prototype, "binaryType");
        binaryTypeOk = desc !== undefined &&
            typeof desc.get === "function" &&
            typeof desc.set === "function";
    } catch (_) { binaryTypeOk = false; }
    results.push(["binarytype-getter-setter", binaryTypeOk]);

    // 6. Method names present on prototype.
    const proto = WebSocket.prototype;
    results.push(["proto-methods-present",
        typeof proto.send === "function" &&
        typeof proto.close === "function" &&
        typeof proto.addEventListener === "function" &&
        typeof proto.removeEventListener === "function"]);

    // 7. Property getters present on prototype (readyState, url, etc.).
    const propsToCheck = ["readyState", "url", "protocol", "extensions", "bufferedAmount"];
    let propsOk = true;
    for (const name of propsToCheck) {
        const desc = Object.getOwnPropertyDescriptor(proto, name);
        if (!desc || typeof desc.get !== "function") { propsOk = false; break; }
    }
    results.push(["proto-properties-present", propsOk]);

    // 8. Symbol.toStringTag or class-name identifies WebSocket.
    results.push(["class-name",
        WebSocket.name === "WebSocket"]);

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
