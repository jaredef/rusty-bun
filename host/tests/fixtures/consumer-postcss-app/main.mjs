import postcss from "postcss";

const css = `
.foo {
  color: red;
  margin: 10px 20px;
}
@media (min-width: 768px) {
  .foo { color: blue; }
}
:hover { cursor: pointer; }
`;

const root = postcss.parse(css);

const rules = [];
root.walkRules(rule => rules.push(rule.selector));

const decls = [];
root.walkDecls(decl => decls.push([decl.prop, decl.value]));

const atRules = [];
root.walkAtRules(at => atRules.push([at.name, at.params]));

// Mutation
root.walkRules(rule => {
  if (rule.selector === ".foo") {
    rule.append({ prop: "padding", value: "5px" });
  }
});

const serialized = root.toString();

process.stdout.write(JSON.stringify({
  rules,
  declCount: decls.length,
  hasColor: decls.some(d => d[0] === "color"),
  atRules,
  hasPaddingAfterMutation: serialized.includes("padding: 5px"),
}) + "\n");
