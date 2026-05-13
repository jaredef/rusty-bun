import { safeLoad } from "yaml-ast-parser";
const r = safeLoad("a: 1\nb: [2, 3]");
process.stdout.write(JSON.stringify({ kind: r.kind, hasMappings: Array.isArray(r.mappings) }) + "\n");
