import { createHooks } from "hookable";
const h = createHooks();
let captured = null;
h.hook("test", v => { captured = v; });
await h.callHook("test", "fired");
process.stdout.write(JSON.stringify({ captured }) + "\n");
