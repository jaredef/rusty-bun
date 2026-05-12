// Brotli decode: encoded "Hello, World!" via Python brotli.compress at
// default level. Bytes hand-verified.
const encoded = new Uint8Array([
  0x0b, 0x06, 0x80, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
  0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x03,
]);

const decoded1 = globalThis.__compression
  ? globalThis.__compression.brotli_decode(Array.from(encoded))
  : null;
const text1 = decoded1 ? new TextDecoder().decode(new Uint8Array(decoded1)) : null;

// Empty stream
const emptyEncoded = new Uint8Array([0x06]);
const decoded2 = globalThis.__compression
  ? globalThis.__compression.brotli_decode(Array.from(emptyEncoded))
  : [];

process.stdout.write(JSON.stringify({
  text1,
  emptyLen: decoded2 ? decoded2.length : -1,
  hasBrotli: !!(globalThis.__compression && typeof globalThis.__compression.brotli_decode === "function"),
}) + "\n");
