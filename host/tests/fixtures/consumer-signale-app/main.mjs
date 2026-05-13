import { Signale } from "signale";
const s = new Signale();
process.stdout.write(JSON.stringify({ hasInfo: typeof s.info, hasError: typeof s.error }) + "\n");
