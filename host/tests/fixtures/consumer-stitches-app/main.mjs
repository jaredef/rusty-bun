// micromustache ^8 — lean mustache subset.
import { render, renderFn } from "micromustache";

const lines = [];
lines.push("1 " + render("Hi {{name}}!", { name: "world" }));
lines.push("2 " + render("{{a.b.c}}", { a: { b: { c: "deep" } } }));
lines.push("3 " + render("{{x}}+{{y}}={{sum}}", { x: 1, y: 2, sum: 3 }));
lines.push("4 " + render("missing={{missing}}", {}));
lines.push("5 " + renderFn("{{x}}+{{y}}", (path, scope) => (scope[path] ?? 0) * 2, { x: 3, y: 4 }));
lines.push("6 " + render("Hello {{user.name}} ({{user.age}})", { user: { name: "alice", age: 30 } }));

process.stdout.write(lines.join("\n") + "\n");
