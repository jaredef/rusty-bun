import { parse, stringify } from "smol-toml";

const lines = [];

// 1: basic parse
{
  const toml = `name = "test"
count = 42
enabled = true`;
  const r = parse(toml);
  lines.push("1 " + JSON.stringify(r));
}

// 2: tables
{
  const toml = `
[server]
host = "localhost"
port = 8080

[database]
url = "postgres://x"
`;
  const r = parse(toml);
  lines.push("2 " + JSON.stringify(r));
}

// 3: arrays
{
  const toml = `items = [1, 2, 3]
names = ["a", "b", "c"]`;
  const r = parse(toml);
  lines.push("3 " + JSON.stringify(r));
}

// 4: array of tables
{
  const toml = `
[[items]]
name = "a"
qty = 1

[[items]]
name = "b"
qty = 2
`;
  const r = parse(toml);
  lines.push("4 " + JSON.stringify(r));
}

// 5: stringify
{
  const obj = { name: "test", count: 10, nested: { a: 1, b: 2 } };
  const out = stringify(obj).trim();
  lines.push("5 " + out.replace(/\n/g, " | "));
}

// 6: round-trip
{
  const orig = `name = "hi"
count = 5
[meta]
created = 1700000000`;
  const parsed = parse(orig);
  const back = parse(stringify(parsed));
  lines.push("6 roundTripEq=" + (JSON.stringify(parsed) === JSON.stringify(back)));
}

// 7: invalid throws
{
  try { parse("name = no quotes"); lines.push("7 NOT_THROWN"); }
  catch (e) { lines.push("7 threw=" + (e instanceof Error)); }
}

process.stdout.write(lines.join("\n") + "\n");
