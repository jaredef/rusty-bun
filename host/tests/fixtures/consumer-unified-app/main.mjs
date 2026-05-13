import { unified } from "unified";
const u = unified();
process.stdout.write(JSON.stringify({ hasUse: typeof u.use, hasParse: typeof u.parse }) + "\n");
