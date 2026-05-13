import * as meriyah from "meriyah";
const ast = meriyah.parseModule("export const x = 42;");
process.stdout.write(JSON.stringify({ type: ast.type, body0: ast.body[0].type }) + "\n");
