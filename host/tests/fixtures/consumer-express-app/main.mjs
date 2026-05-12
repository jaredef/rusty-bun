import express from "express";
import http from "node:http";

const lines = [];
async function main() {
  const app = express();
  app.get("/hello", (req, res) => res.status(200).set("x-from","express").send("hello express"));
  app.get("/json/:name", (req, res) => res.json({greeting:"hi "+req.params.name, ok:true}));
  app.use((req, res) => res.status(404).send("not-found"));
  const server = http.createServer(app);
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
