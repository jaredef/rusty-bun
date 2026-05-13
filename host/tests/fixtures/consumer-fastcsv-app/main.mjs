// fast-csv shape-only (deferred E.29-bis: objectMode + multi-phase
// parser composition in the lib's internals; direct Readable→Transform
// pipe path is fixed, but fast-csv's Parser+HeaderTransformer chain
// surfaces an additional streaming edge that requires per-callback
// tracing).
import { parseString } from "@fast-csv/parse";
process.stdout.write(JSON.stringify({
  hasParseString: typeof parseString === "function",
}) + "\n");
