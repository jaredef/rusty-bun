try {
  const shell = (await import("shelljs")).default;
  process.stdout.write("OK\n");
} catch (e) {
  process.stdout.write("ERR " + (e.stack || e.message) + "\n");
}
