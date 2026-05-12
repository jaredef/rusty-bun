import { parse } from "@babel/parser";

const code1 = `const x = 42; function add(a, b) { return a + b; }`;
const ast1 = parse(code1);

const code2 = `
class Foo extends Bar {
  #private = 1;
  static method() {}
  async asyncMethod() { await Promise.resolve(); }
  *gen() { yield 1; }
}
`;
const ast2 = parse(code2);

const code3 = `import { x, y as z } from "./mod"; export const v = 1; export default 42;`;
const ast3 = parse(code3, { sourceType: "module" });

const code4 = `const t = \`hello \${name} world\`; const arr = [1, 2, ...rest]; const { a, b = 5 } = obj;`;
const ast4 = parse(code4);

process.stdout.write(JSON.stringify({
  ast1Type: ast1.type,
  ast1BodyLen: ast1.program.body.length,
  ast2ClassDecl: ast2.program.body[0].type,
  ast2HasPrivate: !!ast2.program.body[0].body.body.find(m => m.type === "ClassPrivateProperty"),
  ast2MethodNames: ast2.program.body[0].body.body.filter(m => m.type === "ClassMethod").map(m => m.key.name).sort(),
  ast3SourceType: ast3.program.sourceType,
  ast3ImportSpec: ast3.program.body[0].specifiers.map(s => s.local.name).sort(),
  ast4TemplateExpr: ast4.program.body[0].declarations[0].init.type,
  ast4Spread: ast4.program.body[1].declarations[0].init.elements[2].type,
}) + "\n");
