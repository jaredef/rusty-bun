// execa has a deep tail of Node-API surface requirements
// (process.execPath, util.styleText, stream.getDefaultHighWaterMark,
// events.addAbortListener, fs.appendFileSync, etc.). Recorded as
// E.31 execa-deep-tail. Shape-only: report that consumer mode is
// running (Bun reaches the import; rusty-bun reports the gap).
process.stdout.write(JSON.stringify({ probeRan: true }) + "\n");
