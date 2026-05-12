// jsep ^1 — tiny JS expression parser (returns ESTree-like AST).
// Distinct axis: expression-only parsing (no statements), separate from
// acorn (full parser).
import jsep from "jsep";

const lines = [];

function shape(node) {
  if (!node) return null;
  if (Array.isArray(node)) return node.map(shape);
  const out = { type: node.type };
  for (const k of ["operator", "name", "value", "raw", "computed", "prefix"]) {
    if (k in node) out[k] = node[k];
  }
  for (const k of ["left", "right", "argument", "object", "property", "callee", "arguments", "test", "consequent", "alternate", "expressions"]) {
    if (k in node) out[k] = shape(node[k]);
  }
  return out;
}

lines.push("1 " + JSON.stringify(shape(jsep("a + b"))));
lines.push("2 " + JSON.stringify(shape(jsep("foo(1, 2, x)"))));
lines.push("3 " + JSON.stringify(shape(jsep("obj.field"))));
lines.push("4 " + JSON.stringify(shape(jsep("a ? b : c"))));
lines.push("5 " + JSON.stringify(shape(jsep("!x && (y || z)"))));
lines.push("6 " + JSON.stringify(shape(jsep("arr[0] + arr[1]"))));

process.stdout.write(lines.join("\n") + "\n");
