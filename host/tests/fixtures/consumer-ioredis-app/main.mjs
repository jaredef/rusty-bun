try {
  const Redis = (await import("ioredis")).default;
  process.stdout.write(JSON.stringify({
    hasRedis: typeof Redis === "function",
    hasCluster: typeof Redis.Cluster === "function" || typeof Redis.Cluster === "undefined",
  }) + "\n");
} catch (e) {
  process.stdout.write(JSON.stringify({ err: e.name + ": " + e.message.slice(0, 80) }) + "\n");
}
