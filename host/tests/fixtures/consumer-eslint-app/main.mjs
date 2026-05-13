try {
  const m = await import("eslint");
  process.stdout.write(JSON.stringify({
    hasLinter: typeof m.Linter === "function",
    hasESLint: typeof m.ESLint === "function",
    hasRuleTester: typeof m.RuleTester === "function",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 80) }) + "\n");
}
