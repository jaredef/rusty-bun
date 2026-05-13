import equal from "fast-deep-equal";

process.stdout.write(JSON.stringify({
  same: equal({ a: 1, b: [1, 2] }, { a: 1, b: [1, 2] }),
  diff: equal({ a: 1 }, { a: 2 }),
  nested: equal({ x: { y: { z: 1 } } }, { x: { y: { z: 1 } } }),
  nan: equal(NaN, NaN),
  dates: equal(new Date(0), new Date(0)),
  diffDate: equal(new Date(0), new Date(1)),
  arrays: equal([1, [2, [3]]], [1, [2, [3]]]),
}) + "\n");
