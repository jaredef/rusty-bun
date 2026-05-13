import { atom } from "jotai/vanilla";
const a = atom(0);
process.stdout.write(JSON.stringify({ atomType: typeof atom, aInit: a.init, hasRead: typeof a.read }) + "\n");
