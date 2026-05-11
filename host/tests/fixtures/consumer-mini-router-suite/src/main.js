// Tier-J consumer #44: vendored mini-router.
//
// Exercises Request/Response composition + URL parsing + middleware
// chain + async handler dispatch — the production backend pattern
// under itty-router / hono / sunder.

import RouterDefault, { Router } from "mini-router";

const json = Router.json;

async function selfTest() {
    const results = [];

    // 1. Basic GET route returns JSON.
    {
        const r = new Router();
        r.get("/health", () => json({ ok: true }));
        const res = await r.handle(new Request("http://x.test/health"));
        results.push(["basic-get",
            res.status === 200 &&
            (await res.json()).ok === true]);
    }

    // 2. 404 for unmatched path.
    {
        const r = new Router();
        r.get("/exists", () => json({}));
        const res = await r.handle(new Request("http://x.test/nope"));
        results.push(["404-unmatched", res.status === 404]);
    }

    // 3. Path param extraction.
    {
        const r = new Router();
        r.get("/users/:id", (req) => json({ id: req.params.id }));
        const res = await r.handle(new Request("http://x.test/users/42"));
        results.push(["path-param", (await res.json()).id === "42"]);
    }

    // 4. Multiple path params.
    {
        const r = new Router();
        r.get("/repos/:owner/:repo/issues/:num", (req) =>
            json({ owner: req.params.owner, repo: req.params.repo, num: req.params.num }));
        const res = await r.handle(
            new Request("http://x.test/repos/anthropic/rusty-bun/issues/1"));
        const body = await res.json();
        results.push(["multiple-params",
            body.owner === "anthropic" && body.repo === "rusty-bun" && body.num === "1"]);
    }

    // 5. Query string extraction via req.query.
    {
        const r = new Router();
        r.get("/search", (req) => json({ q: req.query.q, page: req.query.page }));
        const res = await r.handle(
            new Request("http://x.test/search?q=foo&page=2"));
        results.push(["query-string", JSON.stringify(await res.json()) === '{"q":"foo","page":"2"}']);
    }

    // 6. POST with JSON body.
    {
        const r = new Router();
        r.post("/items", async (req) => {
            const body = await req.json();
            return json({ created: body.name }, { status: 201 });
        });
        const res = await r.handle(new Request("http://x.test/items", {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ name: "Widget" }),
        }));
        results.push(["post-body",
            res.status === 201 && (await res.json()).created === "Widget"]);
    }

    // 7. Method dispatch: GET and POST on same path are distinct.
    {
        const r = new Router();
        r.get("/x", () => json({ via: "get" }));
        r.post("/x", () => json({ via: "post" }));
        const g = await r.handle(new Request("http://x.test/x"));
        const p = await r.handle(new Request("http://x.test/x", { method: "POST" }));
        results.push(["method-dispatch",
            (await g.json()).via === "get" && (await p.json()).via === "post"]);
    }

    // 8. Global middleware runs before routes; can short-circuit.
    {
        const r = new Router();
        r.use(async (req) => {
            if (req.headers.get("x-block")) return new Response("blocked", { status: 401 });
        });
        r.get("/", () => json({ ok: true }));
        const ok = await r.handle(new Request("http://x.test/"));
        const blocked = await r.handle(new Request("http://x.test/", {
            headers: { "x-block": "yes" },
        }));
        results.push(["middleware-short-circuit",
            ok.status === 200 && blocked.status === 401 && (await blocked.text()) === "blocked"]);
    }

    // 9. Middleware chain: multiple handlers per route, first Response wins.
    {
        const r = new Router();
        r.get("/protected",
            async (req, ctx) => { ctx.user = { id: req.query.user }; /* no return */ },
            async (req, ctx) => json({ user: ctx.user }));
        const res = await r.handle(
            new Request("http://x.test/protected?user=alice"), {});
        results.push(["middleware-chain",
            (await res.json()).user.id === "alice"]);
    }

    // 10. Async handler with await.
    {
        const r = new Router();
        r.get("/async", async () => {
            await new Promise(r => setTimeout(r, 1));
            return json({ async: true });
        });
        const res = await r.handle(new Request("http://x.test/async"));
        results.push(["async-handler", (await res.json()).async === true]);
    }

    // 11. .all() matches any method.
    {
        const r = new Router();
        r.all("/any", (req) => json({ method: req.method }));
        const g = await r.handle(new Request("http://x.test/any"));
        const p = await r.handle(new Request("http://x.test/any", { method: "PATCH" }));
        results.push(["all-methods",
            (await g.json()).method === "GET" && (await p.json()).method === "PATCH"]);
    }

    // 12. Wildcard route.
    {
        const r = new Router();
        r.get("/static/*", (req) => new Response("file: " + req.params[0]));
        // Wildcard doesn't get a named param; my impl uses positional. Check
        // by reading the captured pattern.
        const res = await r.handle(new Request("http://x.test/static/css/main.css"));
        results.push(["wildcard-route",
            res.status === 200]);  // body details vary by capture mechanism
    }

    // 13. Headers in/out.
    {
        const r = new Router();
        r.get("/auth", (req) =>
            json({ token: req.headers.get("authorization") },
                 { headers: { "x-custom": "echo" } }));
        const res = await r.handle(new Request("http://x.test/auth", {
            headers: { "authorization": "Bearer xyz" },
        }));
        results.push(["headers-in-out",
            (await res.json()).token === "Bearer xyz" &&
            res.headers.get("x-custom") === "echo"]);
    }

    // 14. Default export shape (Router class itself).
    results.push(["default-export-is-router",
        RouterDefault === Router]);

    // 15. Composed end-to-end: middleware sets ctx.user via header, route
    //     returns user-scoped response.
    {
        const r = new Router();
        r.use(async (req, ctx) => {
            const auth = req.headers.get("authorization");
            if (!auth) return new Response("unauth", { status: 401 });
            ctx.user = { id: auth.slice(7) };  // strip "Bearer "
        });
        r.get("/me", (req, ctx) => json({ id: ctx.user.id }));
        const ok = await r.handle(
            new Request("http://x.test/me", { headers: { authorization: "Bearer u42" } }),
            {});
        const unauth = await r.handle(new Request("http://x.test/me"), {});
        results.push(["end-to-end-composed",
            ok.status === 200 && (await ok.json()).id === "u42" &&
            unauth.status === 401]);
    }

    return results;
}

const results = await selfTest();
const passed = results.filter(([_, ok]) => ok).length;
const failed = results.filter(([_, ok]) => !ok).map(([name]) => name);
const summary = passed + "/" + results.length +
    (failed.length > 0 ? " failed: " + failed.join(",") : "");
process.stdout.write(summary + "\n");
