import rfdc from "rfdc";

const lines = [];
const clone = rfdc();

// 1: nested object, mutation isolation
{
  const orig = { a: { b: { c: [1, 2, 3] } }, d: "hello" };
  const c = clone(orig);
  c.a.b.c.push(4);
  c.d = "modified";
  lines.push("1 origCLen=" + orig.a.b.c.length + " cloneCLen=" + c.a.b.c.length + " origD=" + orig.d + " cloneD=" + c.d);
}

// 2: arrays
{
  const orig = [[1, 2], [3, 4], [5, 6]];
  const c = clone(orig);
  c[0].push(99);
  lines.push("2 orig0Len=" + orig[0].length + " clone0Len=" + c[0].length);
}

// 3: Date instances
{
  const orig = { ts: new Date(1700000000000), nested: { d: new Date(1800000000000) } };
  const c = clone(orig);
  lines.push("3 sameValue=" + (c.ts.getTime() === orig.ts.getTime()) +
             " differentRef=" + (c.ts !== orig.ts) +
             " isDate=" + (c.ts instanceof Date));
}

// 4: circular refs (with proto:'circles')
{
  const cloneCircles = rfdc({ circles: true });
  const obj = { name: "self" };
  obj.self = obj;
  const c = cloneCircles(obj);
  lines.push("4 selfRefSurvives=" + (c.self === c) + " differentRoot=" + (c !== obj));
}

// 5: primitives passthrough
{
  lines.push("5 " + clone(42) + " " + clone("hi") + " " + clone(null) + " " + clone(true));
}

// 6: prototype chain (default: own props only)
{
  class C { constructor() { this.x = 1; } }
  C.prototype.method = function() { return "proto"; };
  const orig = new C();
  const c = clone(orig);
  lines.push("6 hasX=" + (c.x === 1) + " stillCtor=" + (c instanceof C));
}

process.stdout.write(lines.join("\n") + "\n");
