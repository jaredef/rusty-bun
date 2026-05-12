// handlebars ^4 — templating superset of Mustache. Distinct axis: block
// helpers + compiled template functions.
import Handlebars from "handlebars";

const lines = [];

lines.push("1 " + Handlebars.compile("Hi {{name}}")({ name: "x" }));
lines.push("2 " + Handlebars.compile("{{#each xs}}-{{this}}{{/each}}")({ xs: [1, 2, 3] }));
lines.push("3 " + Handlebars.compile("{{#if a}}A{{else}}B{{/if}}")({ a: false }));
lines.push("4 " + Handlebars.compile("{{#with u}}{{name}}={{age}}{{/with}}")({ u: { name: "alice", age: 30 } }));

Handlebars.registerHelper("upper", s => String(s).toUpperCase());
lines.push("5 " + Handlebars.compile("{{upper s}}")({ s: "hi" }));

lines.push("6 " + Handlebars.compile("{{html}}|{{{html}}}")({ html: "<b>x</b>" }));

process.stdout.write(lines.join("\n") + "\n");
