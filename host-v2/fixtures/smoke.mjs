const sep = path.sep;
const cwd = process.cwd();
const platform = os.platform();
const joined = path.join("a", "b", "c.txt");
const ext = path.extname(joined);

console.log("sep:", sep);
console.log("cwd:", cwd);
console.log("platform:", platform);
console.log("joined:", joined);
console.log("ext:", ext);

Promise.then(Promise.resolve(7), function(x) {
  console.log("promise then:", x * 6);
});
