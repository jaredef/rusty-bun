import getStream from "get-stream";
import { Readable } from "node:stream";
const s = await getStream(Readable.from(["a", "b", "c"]));
process.stdout.write(JSON.stringify({ s }) + "\n");
