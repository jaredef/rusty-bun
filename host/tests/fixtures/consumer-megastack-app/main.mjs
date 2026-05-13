import express from "express";
import bodyParser from "body-parser";
import cors from "cors";
import helmet from "helmet";
import compression from "compression";
import cookieParser from "cookie-parser";

const port = 19000 + (process.pid % 100);
const app = express();

app.use(helmet());
app.use(cors());
app.use(compression());
app.use(cookieParser());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));
app.use((req, res, next) => { req.tag1 = "a"; next(); });
app.use((req, res, next) => { req.tag2 = "b"; next(); });
app.use(async (req, res, next) => { await Promise.resolve(); next(); });
app.use((req, res, next) => { req.tag3 = "c"; next(); });

app.get("/", (req, res) => res.send("hi"));
app.get("/json", (req, res) => res.json({ ok: true, tags: [req.tag1, req.tag2, req.tag3] }));

const server = await new Promise((resolve, reject) => {
  const s = app.listen(port, () => resolve(s));
  s.on("error", reject);
});

const base = `http://127.0.0.1:${port}`;
const r1 = await fetch(`${base}/`);
const r2 = await fetch(`${base}/json`);

process.stdout.write(JSON.stringify({
  r1Status: r1.status,
  r1HasHi: (await r1.text()).includes("hi"),
  r2Status: r2.status,
  r2Body: await r2.json(),
}) + "\n");

server.close();
