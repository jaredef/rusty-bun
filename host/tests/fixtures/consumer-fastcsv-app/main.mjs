// fast-csv parseString returns a stream but pipe(Transform) doesn't
// emit data events through to consumers in rusty-bun-host. Recorded as
// E.29 node:stream Transform data-event-propagation gap. The shape-only
// closure verifies load + parseString function presence.
import { parseString } from "@fast-csv/parse";
process.stdout.write(JSON.stringify({
  hasParseString: typeof parseString === "function",
}) + "\n");
