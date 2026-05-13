import pThrottle from "p-throttle";
const t = pThrottle({ limit: 2, interval: 1000 });
const f = t(async (x) => x * 2);
const r = await Promise.all([f(1), f(2)]);
process.stdout.write(JSON.stringify({ r }) + "\n");
