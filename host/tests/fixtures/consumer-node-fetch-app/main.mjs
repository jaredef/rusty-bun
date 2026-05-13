try {
  const m = await import("node-fetch");
  process.stdout.write(JSON.stringify({
    hasFetch: typeof m.default === "function",
    hasHeaders: typeof m.Headers === "function",
    hasRequest: typeof m.Request === "function",
    hasResponse: typeof m.Response === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 80) }) + "\n");
}
