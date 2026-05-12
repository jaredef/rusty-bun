// mustache ^4 — logic-less templates (real npm lib, distinct from
// in-tree mini suite).
import Mustache from "mustache";

const lines = [];

lines.push("1 " + Mustache.render("Hello, {{name}}!", { name: "world" }));
lines.push("2 " + Mustache.render("{{#items}}-{{.}}{{/items}}", { items: [1, 2, 3] }));
lines.push("3 " + Mustache.render("{{#u}}name={{n}}{{/u}}", { u: { n: "alice" } }));
lines.push("4 " + Mustache.render("{{^empty}}has{{/empty}}{{#empty}}no{{/empty}}", { empty: false }));
lines.push("5 " + Mustache.render("{{html}}|{{{html}}}", { html: "<b>x</b>" }));
lines.push("6 " + Mustache.render("{{a.b.c}}", { a: { b: { c: "deep" } } }));

process.stdout.write(lines.join("\n") + "\n");
