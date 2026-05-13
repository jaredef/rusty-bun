import { watch } from "chokidar";
const w = watch("/tmp", { persistent: false, ignoreInitial: true });
process.stdout.write(JSON.stringify({ hasOn: typeof w.on, hasClose: typeof w.close }) + "\n");
w.close();
