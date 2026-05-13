import { fromMarkdown } from "mdast-util-from-markdown";
const r = fromMarkdown("# H");
process.stdout.write(JSON.stringify({ type: r.type, headDepth: r.children[0].depth }) + "\n");
