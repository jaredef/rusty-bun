// fast-csv shape probe. Direct node:stream Readable→Transform pipe works
// after the double-end fix landed; fast-csv's parseString uses
// objectMode + string_decoder + parser composition that surfaces a
// deeper streaming edge (E.29-bis). Recorded.
import { parseString } from "@fast-csv/parse";
process.stdout.write(JSON.stringify({
  hasParseString: typeof parseString === "function",
}) + "\n");
