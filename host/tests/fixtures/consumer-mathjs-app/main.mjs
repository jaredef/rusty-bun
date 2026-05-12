import { evaluate, parse, sqrt, derivative, simplify } from "mathjs";

const out = {
  add: evaluate("1 + 2"),
  mul: evaluate("3 * 4 - 5"),
  sqrt2: Number(sqrt(2).toFixed(6)),
  expr: parse("2*x + 3").toString(),
  deriv: derivative("x^2 + 2*x", "x").toString(),
  simple: simplify("2*x + x").toString(),
  mat: evaluate("[1, 2] + [3, 4]").valueOf(),
};
process.stdout.write(JSON.stringify(out) + "\n");
