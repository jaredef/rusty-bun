// jsonc-parser ^3 — VSCode's JSONC parser (JSON with comments and
// trailing commas).
import { parse, parseTree, getNodeValue } from "jsonc-parser";

const lines = [];

const errors = [];
const v1 = parse(`{
  // comment
  "a": 1,
  /* block */
  "b": [1, 2, 3,],
  "c": "x",
}`, errors, { disallowComments: false, allowTrailingComma: true });
lines.push("1 " + JSON.stringify(v1) + " errs=" + errors.length);

const t = parseTree('{"x":42}');
lines.push("2 type=" + t.type + " childCount=" + (t.children ? t.children.length : 0));

const v3 = getNodeValue(parseTree('{"a":{"b":[1,2,3]}}'));
lines.push("3 " + JSON.stringify(v3));

const errs4 = [];
parse('{"a": 1,, "b": 2}', errs4);
lines.push("4 hasErrs=" + (errs4.length > 0));

lines.push("5 " + parse('"hello"'));

lines.push("6 " + JSON.stringify(parse('[1, /* mid */ 2, 3]')));

process.stdout.write(lines.join("\n") + "\n");
