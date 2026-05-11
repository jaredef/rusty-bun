import app from "./app.mjs";

const server = Bun.serve({
  port: 0,
  hostname: "127.0.0.1",
  autoServe: true,
  fetch: (req) => app.fetch(req),
});

const port = server.port;
const base = "http://127.0.0.1:" + port;

const lines = [];

{
  const r = await fetch(base + "/");
  lines.push("1 " + r.status + " " + (await r.text()));
}
{
  const r = await fetch(base + "/json/world");
  lines.push("2 " + r.status + " " + (await r.text()));
}
{
  const r = await fetch(base + "/echo", { method: "POST", body: "ping" });
  lines.push("3 " + r.status + " " + (await r.text()));
}
{
  const r = await fetch(base + "/header");
  lines.push("4 " + r.status + " " + r.headers.get("x-custom") + " " + (await r.text()));
}
{
  const r = await fetch(base + "/nope");
  lines.push("5 " + r.status);
}

server.stop();
process.stdout.write(lines.join("\n") + "\n");
