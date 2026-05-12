import { parse } from "acorn";
import escodegen from "escodegen";

const lines = [];

// Parse → regenerate round-trip: a JS-in-JS-in-JS pipeline.
const samples = [
  "var x = 1;",
  "function add(a, b) { return a + b; }",
  "const f = (a, b) => a + b;",
  "class C { method() { return 42; } }",
  "if (x > 0) { y = 1; } else { y = 2; }",
];

for (let i = 0; i < samples.length; i++) {
  const src = samples[i];
  const ast = parse(src, { ecmaVersion: 2024 });
  const out = escodegen.generate(ast).trim();
  lines.push((i + 1) + " " + out.replace(/\s+/g, " "));
}

// Round-trip: parse → generate → parse → check structurally
{
  const src = "const a = [1, 2, 3].map(x => x * 2);";
  const ast1 = parse(src, { ecmaVersion: 2024 });
  const regen = escodegen.generate(ast1);
  const ast2 = parse(regen, { ecmaVersion: 2024 });
  lines.push("6 ast1.body.len=" + ast1.body.length + " ast2.body.len=" + ast2.body.length +
             " bothMatch=" + (ast1.body[0].type === ast2.body[0].type));
}

process.stdout.write(lines.join("\n") + "\n");
