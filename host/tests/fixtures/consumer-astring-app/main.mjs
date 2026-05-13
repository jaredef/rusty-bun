import { generate } from "astring";
const ast = { type: "Program", body: [{ type: "ExpressionStatement", expression: { type: "Literal", value: 42 } }] };
process.stdout.write(JSON.stringify({ r: generate(ast).trim() }) + "\n");
