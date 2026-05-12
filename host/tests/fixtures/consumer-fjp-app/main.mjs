import { applyPatch, compare, observe, generate } from "fast-json-patch";

const lines = [];

// 1: applyPatch
{
  const doc = { a: 1, b: { x: 2 } };
  const patched = applyPatch(doc, [
    { op: "replace", path: "/a", value: 100 },
    { op: "add", path: "/b/y", value: 3 },
    { op: "remove", path: "/b/x" },
  ]).newDocument;
  lines.push("1 " + JSON.stringify(patched));
}

// 2: compare (diff two documents)
{
  const a = { x: 1, y: 2 };
  const b = { x: 1, y: 3, z: 4 };
  const ops = compare(a, b);
  // Sort for deterministic output
  ops.sort((a, b) => a.path.localeCompare(b.path));
  lines.push("2 " + JSON.stringify(ops));
}

// 3: array operations
{
  const doc = { items: ["a", "b", "c"] };
  const patched = applyPatch(doc, [
    { op: "add", path: "/items/-", value: "d" },  // append
    { op: "replace", path: "/items/1", value: "B" },
    { op: "remove", path: "/items/0" },
  ]).newDocument;
  lines.push("3 " + JSON.stringify(patched.items));
}

// 4: test op
{
  const doc = { x: 5 };
  try {
    applyPatch(doc, [{ op: "test", path: "/x", value: 5 }]);
    lines.push("4 pass=true");
  } catch (e) {
    lines.push("4 pass=false");
  }
  try {
    applyPatch(doc, [{ op: "test", path: "/x", value: 99 }]);
    lines.push("4b fail=NOT_THROWN");
  } catch (e) {
    lines.push("4b fail=thrown");
  }
}

// 5: observer + generate (record mutations)
{
  const doc = { count: 0, items: [] };
  const observer = observe(doc);
  doc.count = 5;
  doc.items.push("x");
  const ops = generate(observer);
  lines.push("5 " + JSON.stringify(ops));
}

process.stdout.write(lines.join("\n") + "\n");
