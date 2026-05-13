// Π2.6.c.b empirical demonstration: handler-depth ceiling lifted.
//
// Pre-c.b: fetch's WouldBlock branch yielded exactly 8 microtask
// boundaries per idle spin. A handler with > 8 awaits before
// producing the first response byte exhausted the burst on every
// spin and either stalled or threw "read stalled".
//
// Post-c.b: fetch's WouldBlock branch parks on TCP.waitReadable
// (mio readiness). Handler depth is unbounded; the fetch resumes
// exactly when the response byte appears in the socket buffer.
//
// This fixture exercises a 32-await chain in the handler — four
// times the old 8-burst cap — and asserts the round-trip lands
// byte-identical to Bun.

const port = 0;

const server = Bun.serve({
  port,
  hostname: "127.0.0.1",
  autoServe: true,
  async fetch(req) {
    let count = 0;
    for (let i = 0; i < 32; i++) {
      await Promise.resolve();
      count++;
    }
    return new Response(`depth=${count}`);
  },
});

const r = await fetch(`http://127.0.0.1:${server.port}/`);
const body = await r.text();

process.stdout.write(JSON.stringify({
  status: r.status,
  body,
}) + "\n");

server.stop();
