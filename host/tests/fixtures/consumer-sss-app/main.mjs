import stringify from "safe-stable-stringify";

const lines = [];

// 1: key-order stability
{
  const a = { c: 3, a: 1, b: 2 };
  const b = { a: 1, b: 2, c: 3 };
  lines.push("1 a=" + stringify(a) + " b=" + stringify(b) + " eq=" + (stringify(a) === stringify(b)));
}

// 2: circular refs survive (replaced with "[Circular]")
{
  const obj = { name: "x" };
  obj.self = obj;
  lines.push("2 " + stringify(obj));
}

// 3: nested deterministic
{
  const obj = { z: { c: 3, a: 1 }, a: { y: 2, x: 1 } };
  lines.push("3 " + stringify(obj));
}

// 4: with pretty indent
{
  const out = stringify({ a: 1, b: 2 }, null, 2);
  lines.push("4 " + out.replace(/\n/g, "|"));
}

// 5: typical types
{
  lines.push("5 " + stringify([1, "x", true, null, { n: 1 }, [2, 3]]));
}

process.stdout.write(lines.join("\n") + "\n");
