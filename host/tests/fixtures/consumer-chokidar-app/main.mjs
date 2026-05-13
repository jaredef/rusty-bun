try {
  const m = await import("chokidar");
  process.stdout.write(JSON.stringify({
    hasWatch: typeof m.watch === "function",
    hasFSWatcher: typeof m.FSWatcher === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 80) }) + "\n");
}
