import minimist from "minimist";

const a = minimist(["--name", "ada", "--count", "42", "--ok", "extra"]);
const b = minimist(["-abc", "-d", "value"]);
const c = minimist(["one", "two", "--flag"], { boolean: ["flag"] });
const d = minimist(["--no-bar", "--foo=hello"]);

process.stdout.write(JSON.stringify({
  a: { name: a.name, count: a.count, ok: a.ok, _: a._ },
  b: { a: b.a, b: b.b, c: b.c, d: b.d },
  c: { flag: c.flag, _: c._ },
  d: { bar: d.bar, foo: d.foo },
}) + "\n");
