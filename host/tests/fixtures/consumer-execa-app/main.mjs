try {
  const m = await import("execa");
  process.stdout.write(JSON.stringify({
    hasExeca: typeof m.execa === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 100) }) + "\n");
}
