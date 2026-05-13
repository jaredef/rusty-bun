import { deflateSync, inflateSync, strToU8, strFromU8 } from "fflate";
const c = deflateSync(strToU8("hello hello"));
const d = strFromU8(inflateSync(c));
process.stdout.write(JSON.stringify({ d, cLen: c.length }) + "\n");
