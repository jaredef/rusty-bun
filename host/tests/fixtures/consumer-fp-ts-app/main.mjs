import * as O from "fp-ts/Option";
import { pipe } from "fp-ts/function";
const r = pipe(O.some(2), O.map(x => x * 3));
process.stdout.write(JSON.stringify({ tag: r._tag, val: r.value }) + "\n");
