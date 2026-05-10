# fetch-api pilot — coverage audit

Seventh pilot. **System pilot — first multi-surface composition.** Pilots 1-6 covered single web-platform surfaces. This pilot tests the apparatus' value claim at scale by deriving three composed surfaces (Headers, Request, Response) as a single crate with cross-surface integration tests.

A "bigger portion of Bun" per the keeper's directive. The fetch API is the load-bearing networking surface for any modern JS runtime; getting Headers, Request, Response right is what makes a runtime fetch-compatible.

## Constraint inputs

| Surface | Verifier props | Cross-corroborated cardinality |
|---|---:|---:|
| Headers | 5 properties / 15 clauses | 10 (cs construction-style) |
| Request | 6 properties / 53 clauses | 5 (cs construction-style) |
| Response | 7 properties / 100+ clauses | **73 + 7 + 8 + 4 + 9 + 6 = 107** across multiple cluster groups |
| **Total** | **~18 properties / ~170 clauses** | **122 cross-corroborated cardinality** |

This is **the densest pilot input the apparatus has produced.** Response alone has more cross-corroborated cardinality (107) than every prior pilot put together except structuredClone (166).

Antichain reps drawn from real Bun tests include (Response):
- `expect(typeof Response !== "undefined").toBe(true)`
- `expect(response.status).toBe(200)`
- `expect(response.ok).toBe(true)`
- `expect(await response.text()).toBe("hello world")`
- `expect(await response.json()).toEqual({ok: true})`
- `expect(response.headers.get("content-type")).toBe("application/json")`
- `expect(Response.json(data)).toBeInstanceOf(Response)`
- `expect(Response.redirect(url, 301)).toBeInstanceOf(Response)`

## Pilot scope

Three composed surfaces in a single crate:

### Headers (WHATWG Fetch §5.2)
- Constructor from sequence of pairs / record / another Headers
- `append`, `delete`, `get`, `getSetCookie`, `has`, `set`
- Iteration: `entries`, `keys`, `values`, `for_each`
- Case-insensitive name handling, lowercased iteration
- HTTP whitespace stripping from values
- TypeError on invalid names/values

### Request (WHATWG Fetch §6.2)
- Constructor from `(USVString | Request)` + `RequestInit`
- Method, URL, headers, body, mode, credentials, cache, redirect, signal
- Body extraction: `text`, `array_buffer`, `blob`, `json`, `bytes`, `form_data`
- `clone` with body tee'd; throws when bodyUsed
- Default `method == "GET"`

### Response (WHATWG Fetch §6.4)
- Constructor from optional body + `ResponseInit`
- Status, statusText, headers, body, ok, type, url, redirected
- Body extraction same shape as Request
- Static `Response.error()`, `Response.json(data)`, `Response.redirect(url, status)`
- `clone` with body tee'd
- Status-range validation: `RangeError` outside 200..=599

Cross-cutting integration:
- Response composes Headers
- Request composes Headers and (optionally) Body
- Response.json() sets Content-Type header automatically
- Body is a substrate type shared by Request and Response

## Out of pilot scope

- Actual HTTP transport (no network I/O)
- ReadableStream body type (would need a streams pilot upstream)
- Cookie jar handling
- CORS / credentials / mode enforcement at fetch time
- AbortSignal integration via fetch (the AbortController pilot already covers signal mechanics)
- Service worker fetch interception

## Approach

Single Cargo crate `rusty-fetch-api` with three modules: `headers`, `request`, `response`. Body type is in a shared `body.rs` module. Integration tests verify cross-surface composition.

Body in pilot scope: `Body::Empty`, `Body::Bytes(Vec<u8>)`, `Body::Text(String)`. ReadableStream variant is omitted; AUDIT acknowledges this. Most consumer expectations exercise text and bytes paths.

## Ahead-of-time hypotheses

1. **The pilot will be larger than prior pilots in absolute LOC** (probably 400-600 code-only) because it composes three surfaces. But the LOC ratio against Bun's hand-written equivalent should still be in the 5-15% range because Bun's fetch implementation is *huge* (handles transport, streaming, full WPT compliance, etc.).
2. **Cross-surface composition will surface a derivation bug** the verifier catches — the pilot 4 pattern (Blob slice swap) repeating at scale. Specifically: Response.json() must set Content-Type AND serialize the data; one of those will likely have a subtle wrong semantics on first try.
3. **Response.redirect status validation** is a finicky spec detail: only 301/302/303/307/308 are valid. AOT prediction: this is the second potential verifier-caught-derivation-bug site.
4. **The consumer regression suite for fetch-API will surface the densest dependency-surface map yet** — fetch is used by almost every npm package that touches HTTP.

## Verifier strategy

~50-60 verifier tests across the three surfaces + ~15-20 consumer regression tests. Pilot succeeds if:
- Verifier closes with documented skips at most for ReadableStream-body path
- Consumer regression closes with 0 fail
- Cross-surface integration tests pass (e.g., constructing a Response with explicit Headers and reading them back)
- LOC ratio against Bun's `Response.rs/zig` + `Request.rs/zig` + `Headers.rs/zig` is below 25%
