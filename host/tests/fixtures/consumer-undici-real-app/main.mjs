// undici namespace shape probe. rusty-bun returns inert-stub Proxy
// objects; Bun returns the real undici namespace. Both have fetch
// as a function and named exports as functions or objects.
const undici = await import("undici");
process.stdout.write(JSON.stringify({
  hasFetch: typeof undici.fetch === "function",
  hasAgent: typeof undici.Agent === "function" || typeof undici.Agent === "object",
  hasErrors: typeof undici.errors === "function" || typeof undici.errors === "object",
}) + "\n");
