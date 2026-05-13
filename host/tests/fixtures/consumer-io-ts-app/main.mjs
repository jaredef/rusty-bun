import * as t from "io-ts";
const User = t.type({ name: t.string, age: t.number });
const r = User.decode({ name: "Ada", age: 30 });
process.stdout.write(JSON.stringify({ tag: r._tag }) + "\n");
