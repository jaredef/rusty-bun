// expr-eval ^2 — sandboxed math expression evaluator (distinct from
// mathjs which had its own edges; expr-eval is the slimmer alt).
import { Parser } from "expr-eval";

const lines = [];
const p = new Parser();

lines.push("1 " + p.evaluate("2 + 3 * 4"));
lines.push("2 " + p.evaluate("(1+2) * (3+4)"));
lines.push("3 " + p.evaluate("sin(0) + cos(0)"));
lines.push("4 " + p.parse("x^2 + 2*x + 1").evaluate({ x: 3 }));
lines.push("5 " + p.evaluate("sqrt(16) + abs(-7)"));
lines.push("6 " + p.parse("a > b ? a : b").evaluate({ a: 7, b: 12 }));

process.stdout.write(lines.join("\n") + "\n");
