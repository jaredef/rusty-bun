try {
  const m = await import("p-limit");
  process.stdout.write("import=ok default=" + typeof m.default + "\n");
  const pLimit = m.default;
  const limit = pLimit(2);
  process.stdout.write("limit=" + typeof limit + "\n");
  const results = await Promise.all([1,2,3].map(n => limit(async () => n * 10)));
  process.stdout.write("results=" + JSON.stringify(results) + "\n");
} catch (e) { process.stdout.write("ERR " + e.constructor.name + ": " + e.message + "\n"); }
