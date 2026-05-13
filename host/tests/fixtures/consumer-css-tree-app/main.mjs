import { parse, generate, walk } from "css-tree";

const css = `.foo { color: red; margin: 10px; }
@media (min-width: 768px) { .foo:hover { color: blue; } }`;

const ast = parse(css);
const generated = generate(ast);

const rules = [];
walk(ast, (node) => {
  if (node.type === "Rule") rules.push(generate(node.prelude));
});

process.stdout.write(JSON.stringify({
  astType: ast.type,
  ruleCount: rules.length,
  rulesHaveFoo: rules.some(s => s.includes(".foo")),
  generatedHasColor: generated.includes("color"),
  generatedHasMedia: generated.includes("@media"),
}) + "\n");
