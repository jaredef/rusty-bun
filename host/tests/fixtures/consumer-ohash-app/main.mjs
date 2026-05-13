import { hash } from "ohash";
const h = hash({ a: 1, b: 2 });
process.stdout.write(JSON.stringify({ hasHash: typeof hash, hLen: h.length }) + "\n");
