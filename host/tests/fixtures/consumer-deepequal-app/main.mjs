import deepEqual from "deep-equal";

const out = {
  same: deepEqual({ a: 1, b: [1, 2, 3] }, { a: 1, b: [1, 2, 3] }),
  diff: deepEqual({ a: 1 }, { a: 2 }),
  nested: deepEqual({ a: { b: { c: 1 } } }, { a: { b: { c: 1 } } }),
  nestedDiff: deepEqual({ a: { b: { c: 1 } } }, { a: { b: { c: 2 } } }),
  arrays: deepEqual([1, [2, [3]]], [1, [2, [3]]]),
  arrayDiff: deepEqual([1, 2], [1, 3]),
  // Edge: NaN
  nan: deepEqual(NaN, NaN),
  // Edge: undefined vs null
  undefVsNull: deepEqual(undefined, null),
  // Strict mode
  strictNumStr: deepEqual(1, "1", { strict: true }),
  looseNumStr: deepEqual(1, "1"),
  // Date
  date: deepEqual(new Date(0), new Date(0)),
  dateDiff: deepEqual(new Date(0), new Date(1)),
};

process.stdout.write(JSON.stringify(out) + "\n");
