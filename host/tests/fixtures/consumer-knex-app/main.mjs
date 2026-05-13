// knex SQL builder. Module load works but actual builder use requires
// sqlite3 client native binding (or pg/mysql). Shape-only.
try {
  const k = (await import("knex")).default;
  process.stdout.write(JSON.stringify({
    hasKnex: typeof k === "function" || typeof k === "object",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name }) + "\n");
}
