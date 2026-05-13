import { VFileMessage } from "vfile-message";
const m = new VFileMessage("oops");
process.stdout.write(JSON.stringify({ reason: m.reason }) + "\n");
