import { pack, unpack, Packr, Unpackr } from "msgpackr";

const cases = [
  null,
  true,
  false,
  0,
  -1,
  255,
  65535,
  -2147483648,
  3.14159,
  "hello",
  "café",
  [1, 2, 3],
  { a: 1, b: "two", c: [true, false] },
  { nested: { deep: { value: 42 } } },
];

const results = cases.map((c) => {
  const buf = pack(c);
  const round = unpack(buf);
  return {
    bytes: Array.from(buf),
    round,
    match: JSON.stringify(round) === JSON.stringify(c),
  };
});

const packr = new Packr();
const unpackr = new Unpackr();
const cls = unpackr.unpack(packr.pack({ x: 1, y: [1, 2] }));

process.stdout.write(JSON.stringify({
  results,
  cls,
}) + "\n");
