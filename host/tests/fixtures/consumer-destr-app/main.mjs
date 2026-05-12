// destr ^2 — safer JSON.parse (returns input on parse failure or
// rejects __proto__ etc).
import { destr } from "destr";

const lines = [];
lines.push("1 " + JSON.stringify(destr('{"a":1,"b":2}')));
lines.push("2 " + destr("42"));
lines.push("3 " + destr("true"));
lines.push("4 " + destr("not json"));
lines.push("5 " + destr("null"));
lines.push("6 " + JSON.stringify(destr('[1,"two",3]')));

process.stdout.write(lines.join("\n") + "\n");
