// lodash-es ^4 — ESM build of lodash (distinct from the existing
// consumer-lodash-app which uses the CJS default).
import { chunk, sortBy, groupBy, uniq, zip, mapValues } from "lodash-es";

const lines = [];
lines.push("1 " + JSON.stringify(chunk([1, 2, 3, 4, 5], 2)));
lines.push("2 " + JSON.stringify(sortBy([{ a: 3 }, { a: 1 }, { a: 2 }], "a")));
lines.push("3 " + JSON.stringify(groupBy([1, 2, 3, 4, 5], n => n % 2 === 0 ? "e" : "o")));
lines.push("4 " + JSON.stringify(uniq([1, 2, 1, 3, 2])));
lines.push("5 " + JSON.stringify(zip([1, 2, 3], ["a", "b", "c"])));
lines.push("6 " + JSON.stringify(mapValues({ a: 1, b: 2 }, v => v * 10)));

process.stdout.write(lines.join("\n") + "\n");
