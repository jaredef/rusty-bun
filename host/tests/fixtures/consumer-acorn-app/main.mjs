// acorn ^8 — JS parser. Recursive substrate test: a JS-in-JS parser
// running through QuickJS. Tests ESM + AST emission + parsing arrow
// functions, classes, async, optional chaining, etc.
import { parse, tokenizer } from "acorn";

const lines = [];

// 1: basic expression
{
  const ast = parse("1 + 2 * 3", { ecmaVersion: 2024 });
  lines.push("1 progType=" + ast.type + " bodyLen=" + ast.body.length + " stmt0=" + ast.body[0].type);
}

// 2: variable declaration + parser strict
{
  const ast = parse("const x = 42;", { ecmaVersion: 2024 });
  const decl = ast.body[0];
  lines.push("2 kind=" + decl.kind + " name=" + decl.declarations[0].id.name + " val=" + decl.declarations[0].init.value);
}

// 3: arrow function
{
  const ast = parse("const f = (a, b) => a + b;", { ecmaVersion: 2024 });
  const arrow = ast.body[0].declarations[0].init;
  lines.push("3 type=" + arrow.type + " params=" + arrow.params.length + " expr=" + arrow.expression);
}

// 4: async + optional chaining
{
  const ast = parse("async function f() { return obj?.deep?.value ?? 0; }", { ecmaVersion: 2024 });
  const fn = ast.body[0];
  lines.push("4 fnAsync=" + fn.async + " body=" + fn.body.type);
}

// 5: class with private field
{
  const ast = parse("class C { #x = 1; method() { return this.#x; } }", { ecmaVersion: 2024 });
  lines.push("5 classType=" + ast.body[0].type + " bodyLen=" + ast.body[0].body.body.length);
}

// 6: tokenize a snippet
{
  const tokens = [];
  for (const t of tokenizer("1+2", { ecmaVersion: 2024 })) tokens.push(t.type.label);
  lines.push("6 tokens=" + tokens.join(","));
}

// 7: syntax error
{
  try { parse("const = 1;", { ecmaVersion: 2024 }); lines.push("7 NOT_THROWN"); }
  catch (e) { lines.push("7 threw=" + (e instanceof SyntaxError) + " hasLoc=" + (e.loc !== undefined)); }
}

process.stdout.write(lines.join("\n") + "\n");
