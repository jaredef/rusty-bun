import polka from "polka";

const port = 18900 + (process.pid % 100);

const app = polka()
  .get("/", (req, res) => res.end("hi"))
  .get("/json/:name", (req, res) => {
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify({ name: req.params.name }));
  })
  .post("/echo", async (req, res) => {
    let body = "";
    req.on("data", chunk => body += chunk);
    req.on("end", () => {
      res.setHeader("content-type", "application/json");
      res.end(JSON.stringify({ echoed: body }));
    });
  });

await new Promise(r => app.listen(port, r));

const base = `http://127.0.0.1:${port}`;
const r1 = await fetch(`${base}/`);
const r2 = await fetch(`${base}/json/ada`);

process.stdout.write(JSON.stringify({
  r1: { status: r1.status, body: await r1.text() },
  r2: { status: r2.status, body: await r2.text() },
}) + "\n");

if (app.server && app.server.close) app.server.close();
