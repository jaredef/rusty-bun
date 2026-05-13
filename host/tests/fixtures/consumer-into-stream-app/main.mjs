import intoStream from "into-stream";
import getStream from "get-stream";
const s = await getStream(intoStream("hello"));
process.stdout.write(JSON.stringify({ s }) + "\n");
