import cleanStack from "clean-stack";

const stack = `Error: boom
    at userFn (/home/me/app/index.js:10:5)
    at Module._compile (node:internal/modules/cjs/loader:1234:5)
    at process.processTicksAndRejections (node:internal/process/task_queues:96:5)
    at /home/me/app/node_modules/pirates/lib/index.js:5:1
    at /home/me/app/lib/util.js:3:1`;

const cleaned = cleanStack(stack);
const pretty = cleanStack(stack, { pretty: true });
const based = cleanStack(stack, { basePath: "/home/me/app" });

process.stdout.write(JSON.stringify({
  cleaned: cleaned.split("\n").length,
  hasNodeInternal: cleaned.includes("node:internal"),
  hasPirates: cleaned.includes("pirates"),
  hasUserFn: cleaned.includes("userFn"),
  basedHasFullPath: based.includes("/home/me/app/lib"),
  prettyType: typeof pretty,
}) + "\n");
