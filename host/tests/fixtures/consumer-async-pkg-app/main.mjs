import async from "async";

const map = await async.map([1, 2, 3], (n, cb) => cb(null, n * 2));
const each = [];
await async.each([1, 2, 3], (n, cb) => { each.push(n); cb(); });
const series = await async.series([
  cb => cb(null, "a"),
  cb => cb(null, "b"),
]);
const filtered = await async.filter([1, 2, 3, 4], (n, cb) => cb(null, n % 2 === 0));

process.stdout.write(JSON.stringify({
  map, each: each.sort(), series, filtered,
}) + "\n");
