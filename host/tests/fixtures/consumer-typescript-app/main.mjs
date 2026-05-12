import ts from "typescript";

const code = `
interface User { id: number; name: string; }
function greet(u: User): string { return "hi " + u.name; }
const ada: User = { id: 1, name: "Ada" };
console.log(greet(ada));
`;

const result = ts.transpileModule(code, {
  compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext },
});

// AST visit
const src = ts.createSourceFile("x.ts", code, ts.ScriptTarget.ES2020, true);
const ifaces = [];
const funcs = [];
ts.forEachChild(src, node => {
  if (ts.isInterfaceDeclaration(node)) ifaces.push(node.name.text);
  if (ts.isFunctionDeclaration(node)) funcs.push(node.name.text);
});

process.stdout.write(JSON.stringify({
  hasOutputText: typeof result.outputText === "string",
  outputContainsGreet: result.outputText.includes("function greet"),
  outputDoesntHaveInterface: !result.outputText.includes("interface User"),
  interfaces: ifaces,
  functions: funcs,
  version: typeof ts.version === "string",
}) + "\n");
