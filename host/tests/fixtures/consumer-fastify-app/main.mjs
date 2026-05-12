import Fastify from "fastify";
const lines = [];

const app = Fastify({ logger: false });
lines.push("1 type=" + typeof app);
lines.push("2 hasRoute=" + (typeof app.route === "function"));
lines.push("3 hasGet=" + (typeof app.get === "function"));

app.get("/hello", async (req, reply) => {
  return { msg: "hi", name: req.query.name || "world" };
});
const res = await app.inject({ method: "GET", url: "/hello?name=fastify" });
lines.push("4 status=" + res.statusCode);
lines.push("5 body=" + res.body);

const res2 = await app.inject({ method: "GET", url: "/missing" });
lines.push("6 missing=" + res2.statusCode);

await app.close();

process.stdout.write(lines.join("\n") + "\n");
