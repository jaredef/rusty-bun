import set from "lodash.set";
const o = {};
set(o, "a.b.c", 42);
process.stdout.write(JSON.stringify(o) + "\n");
