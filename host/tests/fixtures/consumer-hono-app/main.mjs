import { Hono } from "hono";

const app = new Hono();
app.get("/", (c) => c.text("hi"));
app.get("/json/:name", (c) => c.json({ name: c.req.param("name") }));
app.post("/echo", async (c) => {
  const body = await c.req.json();
  return c.json({ got: body });
});

const r1 = await app.request("/");
const r2 = await app.request("/json/ada");
const r3 = await app.request("/echo", {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({ a: 1 }),
});

process.stdout.write(JSON.stringify({
  r1: { status: r1.status, body: await r1.text() },
  r2: { status: r2.status, body: await r2.text() },
  r3: { status: r3.status, body: await r3.text() },
}) + "\n");
