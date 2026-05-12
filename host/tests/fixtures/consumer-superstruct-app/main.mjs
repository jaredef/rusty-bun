import { object, string, number, optional, array, refine, define, is, create, validate, StructError } from "superstruct";

const lines = [];

// 1: basic object validation
{
  const U = object({ name: string(), age: number() });
  const r = validate({ name: "alice", age: 30 }, U);
  lines.push("1 ok=" + (r[0] === undefined) + " val=" + JSON.stringify(r[1]));
}

// 2: invalid
{
  const U = object({ name: string(), age: number() });
  const r = validate({ name: "alice", age: "thirty" }, U);
  lines.push("2 hasErr=" + (r[0] !== undefined) + " errType=" + (r[0] && r[0].constructor.name));
}

// 3: array
{
  const A = array(number());
  lines.push("3 ok=" + is([1, 2, 3], A) + " fail=" + is([1, "x", 3], A));
}

// 4: optional + nested
{
  const U = object({ name: string(), tags: optional(array(string())) });
  lines.push("4 a=" + is({ name: "a" }, U) + " b=" + is({ name: "a", tags: ["x"] }, U) + " c=" + is({ name: "a", tags: [1] }, U));
}

// 5: refine — custom validation
{
  const Pos = refine(number(), "positive", v => v > 0 || "must be positive");
  lines.push("5 pos=" + is(5, Pos) + " neg=" + is(-1, Pos));
}

// 6: create with default
{
  const U = object({ name: string(), age: number() });
  try {
    const u = create({ name: "alice", age: 30 }, U);
    lines.push("6 " + JSON.stringify(u));
  } catch (e) { lines.push("6 err=" + e.message); }
}

process.stdout.write(lines.join("\n") + "\n");
