import { match, P } from "ts-pattern";
const r = match({ kind: "ok", value: 42 })
  .with({ kind: "ok" }, x => x.value)
  .otherwise(() => -1);
process.stdout.write(JSON.stringify({ result: r }) + "\n");
