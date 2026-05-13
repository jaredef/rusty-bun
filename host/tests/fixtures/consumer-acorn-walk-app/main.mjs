import { simple } from "acorn-walk";
import { parse } from "acorn";
let count = 0;
simple(parse("1+2", { ecmaVersion: 2022 }), { Literal() { count++; } });
process.stdout.write(JSON.stringify({ count }) + "\n");
