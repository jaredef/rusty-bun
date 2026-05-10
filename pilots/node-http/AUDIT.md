# node-http pilot — coverage audit

**Fifteenth pilot. Tier-C #7 from the trajectory queue.** Node's `http`/`https` module — data-layer only.

## Constraint inputs

`runs/2026-05-10-bun-v0.13b-spec-batch/constraints/http2.constraints.md` covers HTTP/2 (out of pilot scope). The Node-compat http surface is dispersed; primary references are Node's docs §http and Bun's `js/node/http.ts` + `_http_server.ts` + `_http_outgoing.ts` + `_http_incoming.ts`.

## Pilot scope

Data-layer only. No socket binding, no actual wire format. The pilot models *what data structures Node http exposes* — request/response shape, header normalization, status codes, body assembly.

In scope:
- `IncomingMessage` — request on server / response on client. Has `method`, `url`, `headers`, `status_code`, `status_message`, `http_version`, `complete`
- `ServerResponse` — server-side response writer. `write_head`, `set_header`, `get_header`, `write`, `end`, `headers_sent`, `status_code`, `status_message`
- `ClientRequest` — client-side request writer. `write`, `end`, `set_header`, `abort`, `aborted`
- `create_server(handler) -> Server` — creates a server (data-layer only; no listen)
- `Server::on_request(handler)` — primary event hook
- `Server::listen(port)` — sets state but doesn't bind in pilot scope
- `Server::close()` — state transition
- Headers: case-insensitive object-style (Node's flat object representation)

Out of scope:
- Actual transport / socket
- HTTP/2, HTTP/3
- TLS for https (separate pilot or Tier-D)
- Connection pooling
- Streaming body integration with streams pilot (deferred)
- llhttp parser internals (used by Bun for actual parsing)

## LOC budget

Bun reference: `js/node/http.ts` (71) + `_http_server.ts` (1,925) + `_http_outgoing.ts` (588) + `_http_common.ts` (253) + `_http_incoming.ts` (479) = **3,316 LOC** of TS for the node-compat http data-layer. Plus llhttp parser (10,154 LOC C) which is out of pilot scope.

Pilot target: 150-250 code-only LOC for the data-layer shape.

## Approach

Three structs (IncomingMessage, ServerResponse, ClientRequest) + Server. Headers as `Vec<(String, String)>` with case-insensitive lookup methods (matches Node's lowercased `req.headers` flat object).

## Ahead-of-time hypotheses

1. **Pilot is small in LOC.** The data-layer shape is mostly getters/setters. Estimated 150-200 LOC.

2. **First-run clean closure expected.** Node http's data-layer is well-documented; no semantic ambiguity comparable to streams' tee or Blob's slice swap.

3. **The body-as-Vec<u8> simplification** is the load-bearing scope choice. Real Node http uses streaming body via Readable/Writable; pilot accumulates bytes via write/end. Documented in AUDIT as the data-layer adjustment.

## Verifier strategy

~20 verifier tests covering construction, header normalization, write/end semantics, status codes, server/client roundtrip. Consumer regression: ~6-8 tests citing real Node-ecosystem http usage (express, koa, axios server-side, supertest).
