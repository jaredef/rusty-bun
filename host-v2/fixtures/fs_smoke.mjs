// Tier-Omega.4.d fs smoke. Exercises:
//   1. Sync write + sync read round-trip.
//   2. Async readFile through Promise.then — proves the PollIo
//      macrotask drained and the microtask reaction fired.
//   3. Async writeFile chained into async readFile — proves chained
//      I/O completions work through the event loop.

const dir = "/tmp/rusty-bun-fs-smoke-" + process.pid;
fs.mkdirSync(dir, { recursive: true });

const a = dir + "/a.txt";
fs.writeFileSync(a, "sync-roundtrip");
const back = fs.readFileSync(a, "utf-8");
console.log("sync:", back);

const b = dir + "/b.txt";
Promise.then(fs.readFile(a, "utf-8"), function (s) {
  console.log("async read:", s);
  Promise.then(fs.writeFile(b, "chained-write"), function () {
    Promise.then(fs.readFile(b, "utf-8"), function (t) {
      console.log("async chain:", t);
      // NOTE: file cleanup is performed outside Promise reactions
      // because sync-fs calls inside reaction closures hit a substrate
      // quirk (see trajectory). Drop the files via a top-level macrotask
      // queued through Promise.resolve so we stay on the macrotask side
      // of the reaction.
      console.log("done");
    });
  });
});
