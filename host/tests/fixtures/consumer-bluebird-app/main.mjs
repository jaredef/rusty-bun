import Promise from "bluebird";

const r1 = await Promise.resolve(42);
const all = await Promise.all([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)]);
const mapped = await Promise.map([1, 2, 3], n => n * 2);
const filtered = await Promise.filter([1, 2, 3, 4], n => n % 2 === 0);

let caught = null;
try {
  await Promise.reject(new Error("boom"));
} catch (e) { caught = e.message; }

process.stdout.write(JSON.stringify({ r1, all, mapped, filtered, caught }) + "\n");
