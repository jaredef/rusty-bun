import { atom } from "nanostores"; const s = atom(0); s.set(42); process.stdout.write(JSON.stringify({val:s.get(),listen:typeof s.listen}) + "\n");
