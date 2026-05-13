import { ok, err, Result } from "neverthrow";
const r = ok(42);
process.stdout.write(JSON.stringify({ isOk: r.isOk(), val: r.value }) + "\n");
