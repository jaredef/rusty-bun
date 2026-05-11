import app from "./app.mjs";

const server = Bun.serve({
  port: 0, hostname: "127.0.0.1", autoServe: true,
  fetch: (req) => app.fetch(req),
});
const base = "http://127.0.0.1:" + server.port;
const lines = [];

async function main() {
  // 1. health (uses ms)
  {
    const r = await fetch(base + "/health");
    lines.push("1 " + r.status + " " + (await r.text()));
  }
  // 2. unauthenticated note create → 401
  {
    const r = await fetch(base + "/notes", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ title: "hi", body: "x" }),
    });
    lines.push("2 " + r.status + " " + (await r.text()));
  }
  // 3. token
  let token;
  {
    const r = await fetch(base + "/auth/token", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ user: "alice" }),
    });
    const j = JSON.parse(await r.text());
    token = j.token;
    lines.push("3 " + r.status + " ttlMs=" + j.ttlMs + " parts=" + token.split(".").length);
  }
  // 4. invalid body → 422
  {
    const r = await fetch(base + "/notes", {
      method: "POST",
      headers: { "content-type": "application/json", authorization: "Bearer " + token },
      body: JSON.stringify({ title: "", body: "x".repeat(2000) }),
    });
    lines.push("4 " + r.status + " " + (await r.text()));
  }
  // 5. valid create
  {
    const r = await fetch(base + "/notes", {
      method: "POST",
      headers: { "content-type": "application/json", authorization: "Bearer " + token },
      body: JSON.stringify({ title: "first", body: "hello" }),
    });
    const j = JSON.parse(await r.text());
    lines.push("5 " + r.status + " ok=" + j.ok + " idLen=" + j.idLen + " user=" + j.record.user + " title=" + j.record.title);
  }
  // 6. count
  {
    const r = await fetch(base + "/notes/count");
    lines.push("6 " + r.status + " " + (await r.text()));
  }
  // 7. 404
  {
    const r = await fetch(base + "/nope");
    lines.push("7 " + r.status);
  }
  // 8. log lines were emitted in the right shape
  lines.push("8 logs=" + app.logs.length + " firstHasInfo=" + (app.logs[0] || "").includes("[info]"));
}

try { await main(); }
catch (e) { lines.push("FATAL " + e.constructor.name + ": " + e.message); }
finally {
  server.stop();
  process.stdout.write(lines.join("\n") + "\n");
}
