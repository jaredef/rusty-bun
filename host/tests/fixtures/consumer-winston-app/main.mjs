try {
  const m = await import("winston");
  process.stdout.write(JSON.stringify({
    hasCreateLogger: typeof m.createLogger === "function",
    hasFormat: typeof m.format === "object" || typeof m.format === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({
    err: e.name + ": " + e.message,
  }) + "\n");
}
