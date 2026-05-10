# Bun.serve pilot — coverage audit

**Twelfth pilot. Tier-B #4 from the trajectory queue. The flagship Bun API.** This pilot is the final unblock for "Bun's HTTP serving layer at data-layer fidelity" — composes with fetch-api (Pilot 7) and Bun.file (Pilot 11).

## Constraint inputs

From `runs/2026-05-10-bun-v0.13b-spec-batch/constraints/bun.constraints.md` (Bun-cluster):
- Bun.serve appears in many test contexts but the cluster phase correctly attributes most clauses to local server bindings (`const server = Bun.serve(...)` followed by `server.fetch(...)`).
- Direct cross-corroboration on `Bun.serve` itself is sparse (~10–20 clauses); most behavior is anchored at the `server` local-binding level.

This is similar to the streams pilot's situation: the test corpus binds locally and method calls don't re-attribute. The pilot's primary input will be Bun's documented API + the indirect evidence in test patterns like `expect(server.port).toBe(0)`, `expect(server.fetch(req).status).toBe(200)`, and the documented `routes` API.

## Pilot scope

**Data-layer only.** No socket binding, no HTTP wire format, no actual network listening. The pilot models *what response Bun.serve produces given a request* — the routing/dispatch core.

In scope:
- `Bun::serve(options)` → `Server`
  - `options.fetch(request) -> Response` — the catch-all handler
  - `options.routes: HashMap<String, RouteHandler>` — Bun's modern routing API
  - `options.error(err) -> Response` — error handler
  - `options.port`, `options.hostname`, `options.development` — config (stored, not bound)
- `Server::fetch(request) -> Response` — the data-layer core
- `Server::reload(options)` — hot-reload of handler/routes
- `Server::stop()` — transitions to stopped (data-layer state)
- `Server::port()`, `Server::hostname()`, `Server::url()`, `Server::pending_requests()` — getters
- Route matching:
  - Static path: `/health` matches exactly
  - Param: `/users/:id` matches `/users/42` and exposes `id`
  - Method-keyed: `{GET: ..., POST: ...}` dispatches by method
  - Wildcard: `/*` (low priority; deferred)

Out of pilot scope:
- Actual socket binding / `tokio::net` / network listening
- HTTP/1.1 + HTTP/2 + HTTP/3 wire parsing
- WebSocket upgrade (`server.upgrade(req)`) — deferred
- TLS handshake / certificate handling
- Static-file serving via `Bun.file` integration — deferred but trivially composable
- Stream-body request handling beyond the data structure
- Async/Promise handler returns — pilot uses synchronous handlers

## Approach

Single Cargo crate `rusty-bun-serve` with a `Server<H>` parameterized over the handler. Routes stored in a Vec for ordered matching. Path-pattern parsing extracts `:param` segments. Dispatch order: routes first (with method match), then catchall `fetch(req)`, then error handler if either throws.

Composes with `rusty-fetch-api` from Pilot 7 for `Request` and `Response` types.

## Ahead-of-time hypotheses

1. **Pilot is medium-large in LOC** — server config + route table + path matcher + dispatcher. Estimated 200-300 LOC.

2. **Path-pattern matcher is the most likely verifier-bug site.** `:param` capture, trailing-slash handling, and method-priority all have edge cases. AOT prediction: at least one bug surfaces.

3. **Method-keyed routes object** is a Bun innovation (not in any prior framework with the same shape). Pilot's representation: `RouteEntry { pattern, methods: HashMap<Method, Handler> }`. The `{GET, POST}` literal in JS becomes a Rust struct.

4. **First-run will produce a clean closure on the dispatch core, but routes will need iteration.** AOT: ~1 verifier-caught derivation bug.

## Verifier strategy

~25-30 verifier tests covering construction, fetch dispatch, route matching (static / param / method-keyed), error handling, reload, stop, getters.

Consumer regression: ~8-10 tests citing real production Bun.serve usage (Bun docs examples, Hono framework, ElysiaJS framework, third-party Bun routers).
