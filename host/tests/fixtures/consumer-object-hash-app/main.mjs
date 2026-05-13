import hash from "object-hash";
const h = hash({ a: 1, b: [2, 3] });
process.stdout.write(JSON.stringify({ hLen: h.length, t: typeof h }) + "\n");
