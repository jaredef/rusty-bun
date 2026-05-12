// Bun.connect API-surface probe. Full same-process round-trip is gated
// on the E.18/E.19 cooperative-loop class; this fixture verifies the
// surface presence and an error-path: connecting to a port that nothing
// listens on must produce a clear error and emit error/close in order.

const errors = [];
const opens = [];
const closes = [];

try {
  await Bun.connect({
    hostname: "127.0.0.1",
    port: 1,  // Reserved; nothing listens there.
    socket: {
      open(s) { opens.push("open"); },
      data(s, d) {},
      error(s, e) { errors.push(e && (e.code || e.name || "err")); },
      close(s) { closes.push("close"); },
    },
  });
} catch (e) {
  errors.push("caught:" + (e.code || e.name || "err"));
}

process.stdout.write(JSON.stringify({
  hasConnect: typeof Bun.connect === "function",
  errorsEmpty: errors.length === 0,
  errorOrCaughtRaised: errors.length > 0 || closes.length > 0,
}) + "\n");
