try {
  const m = await import("inquirer");
  process.stdout.write(JSON.stringify({
    hasPrompt: typeof m.default.prompt === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 100) }) + "\n");
}
