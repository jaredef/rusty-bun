import Koa from "koa";
import http from "node:http";

const lines = [];
async function main() {
  const app = new Koa();
  app.use(async (ctx, next) => {
    if (ctx.path === "/hello") { ctx.status = 200; ctx.set("x-from", "koa"); ctx.body = "hello koa"; return; }
    if (ctx.path.startsWith("/json/")) { ctx.body = { greeting: "hi " + ctx.path.slice(6), ok: true }; return; }
    ctx.status = 404; ctx.body = "not-found";
  });
  const server = http.createServer(app.callback());
  server.listen(0, "127.0.0.1");
  const port = server.address().port;
  const base = "http://127.0.0.1:" + port;
  { const r = await fetch(base + "/hello"); lines.push("1 " + r.status + " " + r.headers.get("x-from") + " " + (await r.text())); }
  { const r = await fetch(base + "/json/world"); lines.push("2 " + r.status + " " + (await r.text())); }
  { const r = await fetch(base + "/nope"); lines.push("3 " + r.status + " " + (await r.text())); }
  server.close();
}
try { await main(); process.stdout.write(lines.join("\n") + "\n"); }
catch (e) { process.stdout.write("FATAL " + e.constructor.name + ": " + e.message + "\n"); }
