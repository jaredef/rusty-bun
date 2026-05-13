import chalkTemplate from "chalk-template";
const out = chalkTemplate`Hello {bold world}`;
process.stdout.write(JSON.stringify({ hasOutput: typeof out === "string", contains: out.includes("world") }) + "\n");
