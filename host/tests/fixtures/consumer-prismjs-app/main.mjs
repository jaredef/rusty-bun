const Prism = (await import("prismjs")).default;
const code = "const x = 42; // hi";
const html = Prism.highlight(code, Prism.languages.javascript, "javascript");
process.stdout.write(JSON.stringify({
  hasHighlight: typeof Prism.highlight === "function",
  hasJs: !!Prism.languages.javascript,
  htmlLen: html.length,
  hasTokens: html.includes("token"),
}) + "\n");
