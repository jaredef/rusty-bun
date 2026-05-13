import pFinally from "p-finally";
let called = false;
const r = await pFinally(Promise.resolve("ok"), () => { called = true; });
process.stdout.write(JSON.stringify({ r, called }) + "\n");
