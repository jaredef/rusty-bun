// ejs ^3 — Embedded JavaScript templates. Distinct axis: inline <% %>
// JS evaluation inside template strings (vs declarative mustache/hbs).
import ejs from "ejs";

const lines = [];

lines.push("1 " + ejs.render("Hi <%= name %>", { name: "world" }));
lines.push("2 " + ejs.render("<% for (let i = 0; i < n; i++) { %>-<%= i %><% } %>", { n: 3 }));
lines.push("3 " + ejs.render("<% if (a) { %>A<% } else { %>B<% } %>", { a: false }));
lines.push("4 " + ejs.render("<%- raw %>|<%= raw %>", { raw: "<b>x</b>" }));
lines.push("5 " + ejs.render("<%= xs.map(x => x*2).join(',') %>", { xs: [1, 2, 3] }));
lines.push("6 " + ejs.render("<%= JSON.stringify(o) %>", { o: { a: 1, b: 2 } }));

process.stdout.write(lines.join("\n") + "\n");
