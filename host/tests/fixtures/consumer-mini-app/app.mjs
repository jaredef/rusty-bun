// Real-shape mini API server composing 6 vendored OSS libs:
//   Bun.serve  → HTTP server (+ Π2.6.b autoServe self-fetch path)
//   itty-router → routing
//   zod         → request body schema validation
//   jose        → HS256 JWT issuance + verification
//   ulid        → sortable request IDs / record IDs
//   ms          → human-readable TTL parsing
//   picocolors  → log line coloring (non-TTY → no codes, byte-stable)
import { Router } from "itty-router";
import { z } from "zod";
import { SignJWT, jwtVerify } from "jose";
import { ulid } from "ulid";
import ms from "ms";
import pc from "picocolors";

const SECRET = new TextEncoder().encode("mini-app-shared-secret-32-bytes");
const TOKEN_TTL_MS = ms("1h");
const EXP = 9999999999;

const CreateNote = z.object({
  title: z.string().min(1).max(80),
  body: z.string().max(1000),
});

// In-process datastore.
const notes = new Map();
const logs = [];

function logAt(level, line) {
  // pc is non-TTY in test → emits plain text. Deterministic.
  const ts = "1700000000000";  // fixed for differential
  const colored = level === "err" ? pc.red(line) : pc.green(line);
  logs.push("[" + ts + "][" + level + "] " + colored);
}

const router = Router();

router.post("/auth/token", async (req) => {
  const body = await req.json().catch(() => ({}));
  if (typeof body.user !== "string" || !body.user) {
    return new Response(JSON.stringify({ error: "user required" }), { status: 400 });
  }
  const tok = await new SignJWT({ sub: body.user })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt(1700000000)
    .setExpirationTime(EXP)
    .sign(SECRET);
  logAt("info", "token issued for " + body.user);
  return new Response(JSON.stringify({ token: tok, ttlMs: TOKEN_TTL_MS }), { status: 200 });
});

router.post("/notes", async (req) => {
  const auth = req.headers.get("authorization") || "";
  const m = auth.match(/^Bearer\s+(.+)$/);
  if (!m) {
    logAt("err", "missing token");
    return new Response(JSON.stringify({ error: "missing token" }), { status: 401 });
  }
  let payload;
  try { ({ payload } = await jwtVerify(m[1], SECRET)); }
  catch { return new Response(JSON.stringify({ error: "bad token" }), { status: 401 }); }
  const raw = await req.json().catch(() => null);
  const parsed = CreateNote.safeParse(raw);
  if (!parsed.success) {
    return new Response(JSON.stringify({ error: "validation", issues: parsed.error.issues.length }), { status: 422 });
  }
  const id = ulid(1700000000000);  // fixed ts → deterministic-shape but byte-stable
  const record = { id, user: payload.sub, ...parsed.data };
  notes.set(id, record);
  logAt("info", "note " + id.slice(0, 6) + "… created");
  return new Response(JSON.stringify({ ok: true, idLen: id.length, record }), { status: 201 });
});

router.get("/notes/count", () =>
  new Response(JSON.stringify({ count: notes.size }), { status: 200 }));

router.get("/health", () =>
  new Response(JSON.stringify({ status: "ok", ttl: ms(TOKEN_TTL_MS) }), { status: 200 }));

const app = {
  fetch: async (req) =>
    (await router.fetch(req)) ?? new Response("not found", { status: 404 }),
  logs,
};

export default app;
