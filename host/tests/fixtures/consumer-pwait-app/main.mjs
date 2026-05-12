import delay from "delay";
const lines = [];

const start = Date.now();
await delay(30);
lines.push("1 elapsed_ok=" + (Date.now() - start >= 25));

const v = await delay(10, { value: "ok" });
lines.push("2 v=" + v);

const controller = new AbortController();
setTimeout(() => controller.abort(), 5);
let err = null;
try {
  await delay(100, { signal: controller.signal });
} catch (e) { err = e.name; }
lines.push("3 abort=" + err);

lines.push("4 isFn=" + (typeof delay === "function"));

await delay(5);
lines.push("5 sequential=ok");

await delay(10);
lines.push("6 second=ok");

process.stdout.write(lines.join("\n") + "\n");
