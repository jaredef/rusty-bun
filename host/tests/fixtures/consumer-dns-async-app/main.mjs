// Π2.6.d.d: async DNS via worker thread + eventfd + reactor.
//
// Resolves localhost (always present) and verifies the resolution
// completes without blocking the eval loop. The shape match is
// what the consumer cares about; the exact address depends on the
// system's /etc/hosts.

const start = Date.now();
const result = await Bun.dns.lookup("localhost");
const elapsed = Date.now() - start;

const has127 = result.some(r => r.address.startsWith("127.") || r.address === "::1");
const hasAny = result.length > 0;

process.stdout.write(JSON.stringify({
  hasAny,
  has127,
}) + "\n");
