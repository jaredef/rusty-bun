import { parse, stringify } from "flatted";
const lines = [];

// 1: simple
{ const s = stringify({ a: 1, b: 2 }); lines.push("1 " + s + " back=" + JSON.stringify(parse(s))); }

// 2: cycle preserved
{
  const a = { n: 1 }; a.self = a;
  const s = stringify(a);
  const back = parse(s);
  lines.push("2 selfRef=" + (back.self === back) + " n=" + back.n);
}

// 3: mutual ref
{
  const a = {}; const b = {}; a.b = b; b.a = a;
  const s = stringify({ a, b });
  const back = parse(s);
  lines.push("3 aTob=" + (back.a.b === back.b) + " bToa=" + (back.b.a === back.a));
}

// 4: arrays
{
  const xs = [1, 2, 3]; xs.push(xs);
  const back = parse(stringify(xs));
  lines.push("4 first3=" + back.slice(0, 3) + " selfAt3=" + (back[3] === back));
}

// 5: typical types
{
  lines.push("5 " + parse(stringify([1, "x", true, null, { n: 1 }, [2, 3]])).join("|"));
}

process.stdout.write(lines.join("\n") + "\n");
