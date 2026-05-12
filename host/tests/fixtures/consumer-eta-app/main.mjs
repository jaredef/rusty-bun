import { Eta } from "eta";

const lines = [];

const eta = new Eta({ useWith: true });

// 1: simple interpolation
{
  const out = eta.renderString("Hello <%= name %>!", { name: "alice" });
  lines.push("1 " + out);
}

// 2: conditional
{
  const tpl = "<% if (n > 0) { %>positive<% } else { %>non-positive<% } %>";
  lines.push("2 " + eta.renderString(tpl, { n: 5 }) + " | " +
              eta.renderString(tpl, { n: -3 }));
}

// 3: loop
{
  const tpl = "<% for (const x of items) { %>[<%= x %>]<% } %>";
  lines.push("3 " + eta.renderString(tpl, { items: ["a", "b", "c"] }));
}

// 4: HTML escaping
{
  const tpl = "<%= s %> vs <%~ s %>";
  lines.push("4 " + eta.renderString(tpl, { s: "<script>" }));
}

// 5: nested object
{
  const tpl = "user=<%= user.name %> age=<%= user.profile.age %>";
  lines.push("5 " + eta.renderString(tpl, { user: { name: "alice", profile: { age: 30 } } }));
}

// 6: include via function
{
  const tpl = "<%= it.greet() %>";
  lines.push("6 " + eta.renderString(tpl, { greet: () => "hello world" }));
}

process.stdout.write(lines.join("\n") + "\n");
