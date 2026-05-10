# Response — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: response-surface-property
  threshold: RESP1
  interface: [Response, Response, Response, Response.prototype.clone, Response.redirect]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 5; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 28.

## RESP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 10 constraint clauses across 3 test files. Antichain representatives:

- `tests/unit/response_test.ts:73` —  → `assertEquals(response.status, 200)`
- `tests/unit/fetch_test.ts:1194` —  → `assertEquals(response.status, 200)`
- `response.spec.md:8` — Response is exposed as a global constructor → `new Response() returns a 200 Response with empty body`

## RESP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:13` — Response constructor validates init → `Response constructor throws RangeError when init.status is outside 200..=599`
- `response.spec.md:14` — Response constructor validates init → `Response constructor throws TypeError when init.statusText contains forbidden characters`

## RESP3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:7` — Response is exposed as a global constructor → `Response is defined as a global constructor in any execution context with [Exposed=*]`

## RESP4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.prototype.clone** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:64` — Response.prototype.clone → `Response.prototype.clone throws TypeError when bodyUsed is true`

## RESP5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:28` — Response.redirect static method → `Response.redirect throws RangeError when status is not 301, 302, 303, 307, or 308`

## RESP6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — satisfies the documented invariant. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `tests/unit/serve_test.ts:668` —  → `assert(response.bodyUsed)`
- `tests/unit/response_test.ts:98` —  → `assert(response.bodyUsed)`
- `response.spec.md:9` — Response is exposed as a global constructor → `new Response(body) wraps body as the response body`

## RESP7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/fetch_test.ts:974` —  → `assertEquals(redir.status, 301)`
- `response.spec.md:26` — Response.redirect static method → `Response.redirect(url) returns a Response with the Location header set to url`
- `tests/unit/fetch_test.ts:975` —  → `assertEquals(redir.statusText, "")`

