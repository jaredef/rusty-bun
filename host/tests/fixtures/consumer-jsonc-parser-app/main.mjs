import { parse } from "jsonc-parser";
const r = parse('{ /* c */ "a": 1, "b": [2, 3] }');
process.stdout.write(JSON.stringify(r) + "\n");
