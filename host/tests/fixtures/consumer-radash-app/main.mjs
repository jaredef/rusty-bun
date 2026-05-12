// radash ^12 — functional utility lib (modern lodash replacement).
import * as _ from "radash";

const lines = [];
lines.push("1 " + JSON.stringify(_.group([1, 2, 3, 4, 5], n => n % 2 === 0 ? "even" : "odd")));
lines.push("2 " + JSON.stringify(_.unique([1, 2, 1, 3, 2, 4])));
lines.push("3 " + JSON.stringify(_.zip([1, 2, 3], ["a", "b", "c"])));
lines.push("4 " + JSON.stringify(_.cluster([1, 2, 3, 4, 5, 6, 7], 3)));
lines.push("5 " + _.title("hello_world-foo bar"));
lines.push("6 " + JSON.stringify(_.objectify([{ k: "a", v: 1 }, { k: "b", v: 2 }], o => o.k, o => o.v)));

process.stdout.write(lines.join("\n") + "\n");
