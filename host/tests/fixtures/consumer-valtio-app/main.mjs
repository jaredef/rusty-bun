import { proxy } from "valtio/vanilla";
const s = proxy({ count: 0 });
s.count = 42;
process.stdout.write(JSON.stringify({ proxyType: typeof proxy, count: s.count }) + "\n");
