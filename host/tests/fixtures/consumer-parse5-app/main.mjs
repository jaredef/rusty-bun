import { parse } from "parse5";
const d = parse("<p>Hi</p>");
process.stdout.write(JSON.stringify({ nodeName: d.nodeName, hasChildren: Array.isArray(d.childNodes) }) + "\n");
