// highlight.js ^11 — syntax highlighter. Distinct axis: language
// tokenization + HTML emission for many programming languages.
import hljs from "highlight.js";

const lines = [];

// 1: highlight JS code, auto-detect not specified
{
  const r = hljs.highlight("const x = 1 + 2;", { language: "javascript" });
  // Just check shape; full HTML output is too large to compare across versions
  // but the structural fields are stable.
  lines.push("1 lang=" + r.language + " relevance=" + (typeof r.relevance) +
             " hasValue=" + (typeof r.value === "string" && r.value.length > 0) +
             " hasKw=" + r.value.includes("hljs-keyword"));
}

// 2: highlight Python
{
  const r = hljs.highlight("def foo():\n  return 1", { language: "python" });
  lines.push("2 lang=" + r.language + " hasKw=" + r.value.includes("hljs-keyword"));
}

// 3: highlight bash
{
  const r = hljs.highlight("echo $HOME", { language: "bash" });
  lines.push("3 lang=" + r.language + " hasBuiltin=" + (r.value.includes("hljs-built_in") || r.value.includes("hljs-variable")));
}

// 4: list registered languages (subset stable across versions)
{
  const langs = hljs.listLanguages().sort();
  // Check a few we expect to be present.
  lines.push("4 hasJs=" + langs.includes("javascript") +
             " hasPy=" + langs.includes("python") +
             " hasBash=" + langs.includes("bash") +
             " hasRust=" + langs.includes("rust"));
}

// 5: highlightAuto with multiple languages
{
  const r = hljs.highlightAuto("function hello() { return 42; }", ["javascript", "python", "ruby"]);
  lines.push("5 detected=" + r.language);
}

// 6: empty input
{
  const r = hljs.highlight("", { language: "javascript" });
  lines.push("6 emptyVal=" + (r.value === ""));
}

process.stdout.write(lines.join("\n") + "\n");
