import { type } from "arktype";
const User = type({ name: "string", age: "number" });
const r = User({ name: "Ada", age: 30 });
process.stdout.write(JSON.stringify({ ok: !(r instanceof type.errors), name: r.name }) + "\n");
