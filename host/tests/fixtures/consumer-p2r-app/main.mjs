import { pathToRegexp, match, compile } from "path-to-regexp";

const lines = [];

// 1: simple
{
  const { regexp, keys } = pathToRegexp("/users/:id");
  lines.push("1 keys=" + JSON.stringify(keys) + " m=" + (regexp.test("/users/42") ? "yes" : "no"));
}

// 2: match parsed params
{
  const m = match("/users/:id");
  const r = m("/users/42");
  lines.push("2 ok=" + (r !== false) + " params=" + JSON.stringify(r.params));
}

// 3: multiple params
{
  const m = match("/orgs/:org/repos/:repo");
  const r = m("/orgs/acme/repos/widget");
  lines.push("3 " + JSON.stringify(r.params));
}

// 4: compile (reverse direction)
{
  const c = compile("/users/:id");
  lines.push("4 " + c({ id: "42" }));
}

// 5: optional segment with {:foo}
{
  const m = match("/users{/:id}");
  lines.push("5 noid=" + (!!m("/users")) + " withid=" + JSON.stringify(m("/users/42").params));
}

process.stdout.write(lines.join("\n") + "\n");
