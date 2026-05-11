// Composed fixture: itty-router + zod + jose on Bun.serve, fetched
// same-process via Π2.6.b. Real-shape mini API server with JWT auth
// + schema validation. The strongest single piece of telos evidence
// to date: three vendored OSS libs orchestrated in one process,
// byte-identical to Bun across the full request lifecycle.

import { Router } from "itty-router";
import { z } from "zod";
import { SignJWT, jwtVerify } from "jose";

const secret = new TextEncoder().encode("stack-shared-secret-32-bytes-x");
const exp = 9999999999;

const CreateBody = z.object({
  title: z.string().min(1).max(100),
  qty: z.number().int().positive(),
});

const router = Router();

router.post("/auth/token", async (req) => {
  const body = await req.json().catch(() => ({}));
  if (typeof body.user !== "string" || body.user.length === 0) {
    return new Response(JSON.stringify({ error: "user required" }), { status: 400 });
  }
  const token = await new SignJWT({ sub: body.user })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt(1700000000)
    .setExpirationTime(exp)
    .sign(secret);
  return new Response(JSON.stringify({ token }), {
    status: 200, headers: { "content-type": "application/json" },
  });
});

router.post("/items", async (req) => {
  const auth = req.headers.get("authorization") || "";
  const m = auth.match(/^Bearer\s+(.+)$/);
  if (!m) return new Response(JSON.stringify({ error: "missing token" }), { status: 401 });
  let payload;
  try { ({ payload } = await jwtVerify(m[1], secret)); }
  catch { return new Response(JSON.stringify({ error: "invalid token" }), { status: 401 }); }
  const raw = await req.json().catch(() => null);
  const parsed = CreateBody.safeParse(raw);
  if (!parsed.success) {
    return new Response(JSON.stringify({ error: "validation", issues: parsed.error.issues.length }), { status: 422 });
  }
  return new Response(JSON.stringify({ ok: true, user: payload.sub, item: parsed.data }), {
    status: 201, headers: { "content-type": "application/json" },
  });
});

router.get("/health", () => new Response(JSON.stringify({ status: "ok" }), { status: 200 }));

const app = {
  fetch: async (req) => (await router.fetch(req)) ?? new Response("not found", { status: 404 }),
};

const server = Bun.serve({
  port: 0, hostname: "127.0.0.1", autoServe: true,
  fetch: (req) => app.fetch(req),
});
const port = server.port;
const base = "http://127.0.0.1:" + port;

const lines = [];
async function main() {
  // 1. health
  {
    const r = await fetch(base + "/health");
    lines.push("1 " + r.status + " " + (await r.text()));
  }
  // 2. unauthenticated create -> 401
  {
    const r = await fetch(base + "/items", { method: "POST", body: JSON.stringify({ title: "x", qty: 1 }) });
    lines.push("2 " + r.status + " " + (await r.text()));
  }
  // 3. obtain token
  let token;
  {
    const r = await fetch(base + "/auth/token", { method: "POST", body: JSON.stringify({ user: "alice" }) });
    const j = JSON.parse(await r.text());
    token = j.token;
    lines.push("3 " + r.status + " parts=" + token.split(".").length);
  }
  // 4. invalid body -> 422
  {
    const r = await fetch(base + "/items", {
      method: "POST",
      headers: { "authorization": "Bearer " + token, "content-type": "application/json" },
      body: JSON.stringify({ title: "", qty: -1 }),
    });
    lines.push("4 " + r.status + " " + (await r.text()));
  }
  // 5. valid request -> 201
  {
    const r = await fetch(base + "/items", {
      method: "POST",
      headers: { "authorization": "Bearer " + token, "content-type": "application/json" },
      body: JSON.stringify({ title: "widget", qty: 3 }),
    });
    lines.push("5 " + r.status + " " + (await r.text()));
  }
  // 6. unmatched route -> 404
  {
    const r = await fetch(base + "/missing");
    lines.push("6 " + r.status);
  }
}

try {
  await main();
} catch (e) {
  lines.push("FATAL " + e.constructor.name + ": " + e.message);
} finally {
  server.stop();
  process.stdout.write(lines.join("\n") + "\n");
}
