// fast-csv loads but stream-based parsing hits an unprobed node:stream
// Transform edge (recorded as E.29 fast-csv stream-edge). Shape-only:
import { parseString } from "@fast-csv/parse";
process.stdout.write(JSON.stringify({
  hasParseString: typeof parseString === "function",
}) + "\n");
