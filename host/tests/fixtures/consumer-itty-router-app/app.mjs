import { Router } from "itty-router";

const router = Router();

router.get("/", () => new Response("hello from itty-router", { status: 200 }));

router.get("/json/:name", ({ params }) =>
  new Response(JSON.stringify({ greeting: "hi " + params.name, ok: true }), {
    status: 200,
    headers: { "content-type": "application/json" },
  })
);

router.post("/echo", async (req) => {
  const body = await req.text();
  return new Response("echo:" + body, { status: 201 });
});

router.get("/header", () =>
  new Response("with-custom-header", {
    status: 200,
    headers: { "x-custom": "itty-set" },
  })
);

// itty-router returns undefined for unmatched; we coerce to 404.
const app = {
  fetch: async (req) => {
    const r = await router.fetch(req);
    return r ?? new Response("not found", { status: 404 });
  },
};

export default app;
