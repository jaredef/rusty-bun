try {
  const m = await import("lit");
  process.stdout.write(JSON.stringify({
    hasLitElement: typeof m.LitElement === "function",
    hasHtml: typeof m.html === "function",
    hasCss: typeof m.css === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 80) }) + "\n");
}
