import * as t from "runtypes";
const User = t.Object({ name: t.String, age: t.Number });
const r = User.check({ name: "Ada", age: 30 });
process.stdout.write(JSON.stringify({ name: r.name }) + "\n");
