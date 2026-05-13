import pDefer from "p-defer";
const d = pDefer();
setTimeout(() => d.resolve(42), 10);
const v = await d.promise;
process.stdout.write(JSON.stringify({ v }) + "\n");
