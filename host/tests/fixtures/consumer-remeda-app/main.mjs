// remeda ^2 — TS-first functional utility lib (separate axis from radash/lodash).
import * as R from "remeda";

const lines = [];
lines.push("1 " + JSON.stringify(R.pipe([1, 2, 3, 4, 5], R.map(n => n * 2), R.filter(n => n > 4))));
lines.push("2 " + JSON.stringify(R.groupBy([1, 2, 3, 4, 5], n => n % 2 === 0 ? "e" : "o")));
lines.push("3 " + JSON.stringify(R.unique([1, 2, 1, 3, 2])));
lines.push("4 " + JSON.stringify(R.zip([1, 2], ["a", "b"])));
lines.push("5 " + JSON.stringify(R.chunk([1, 2, 3, 4, 5], 2)));
lines.push("6 " + JSON.stringify(R.sortBy([{ x: 3 }, { x: 1 }, { x: 2 }], o => o.x)));

process.stdout.write(lines.join("\n") + "\n");
