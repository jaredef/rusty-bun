import fg from "fast-glob";
process.stdout.write(JSON.stringify({ hasFg: typeof fg, hasSync: typeof fg.sync }) + "\n");
