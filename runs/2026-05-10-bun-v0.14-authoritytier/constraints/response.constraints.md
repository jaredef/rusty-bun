# Response — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: response-surface-property
  threshold: RESP1
  interface: [Response, Response.redirect, Response.error, Response, Response, Response, Response.prototype, Response.prototype.clone, Response.redirect]

@imports: []

@pins: []

Surface drawn from 12 candidate properties across the Bun test corpus. Construction-style: 9; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 174.

## RESP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 73 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/19850/19850.test.ts:22` — when beforeEach callback throws > test name is not garbled → `expect(err).toBe(' err-in-hook-and-multiple-tests.ts: 1 | import { beforeEach, test } from "bun:test"; 2 | 3 | beforeEach(() => { 4 | throw new Error("beforeEach"); ^ error: beforeEach at <anonymous> …`
- `test/regression/issue/09555.test.ts:154` — #09555 > Readable.fromWeb consumes the ReadableStream → `expect(response.bodyUsed).toBe(false)`
- `test/js/web/streams/streams.test.js:496` — new Response(stream).body → `expect(response.body).toBe(stream)`

## RESP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 8 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/response.test.ts:99` — Response.redirect status code validation → `expect(Response.redirect("url", 301).status).toBe(301)`
- `test/js/web/fetch/fetch.test.ts:1226` — Response > Response.redirect > works → `expect(Response.redirect(input).headers.get("Location")).toBe(input)`
- `response.spec.md:26` — Response.redirect static method → `Response.redirect(url) returns a Response with the Location header set to url`

## RESP3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.error** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1248` — Response > Response.error > works → `expect(Response.error().type).toBe("error")`
- `response.spec.md:17` — Response.error static method → `Response.error returns a network-error Response with type "error"`
- `test/js/web/fetch/fetch.test.ts:1249` — Response > Response.error > works → `expect(Response.error().ok).toBe(false)`

## RESP4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 2 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:13` — Response constructor validates init → `Response constructor throws RangeError when init.status is outside 200..=599`
- `response.spec.md:14` — Response constructor validates init → `Response constructor throws TypeError when init.statusText contains forbidden characters`

## RESP5
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:79` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Response("")).toBeInstanceOf(Response)`

## RESP6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:7` — Response is exposed as a global constructor → `Response is defined as a global constructor in any execution context with [Exposed=*]`

## RESP7
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-fetch.test.js:19` — node-fetch → `expect(Response.prototype).toBeInstanceOf(globalThis.Response)`

## RESP8
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.prototype.clone** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:64` — Response.prototype.clone → `Response.prototype.clone throws TypeError when bodyUsed is true`

## RESP9
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `response.spec.md:28` — Response.redirect static method → `Response.redirect throws RangeError when status is not 301, 302, 303, 307, or 308`

## RESP10
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Response** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 68)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/body.test.ts:629` — new Response() → `expect(result).toHaveProperty("bodyUsed", false)`
- `test/cli/update_interactive_formatting.test.ts:789` — bun update --interactive > should handle catalog updates in workspaces.catalogs object → `expect(output).toContain("Installing updates...")`
- `test/cli/install/bun-update.test.ts:72` — should update to latest version of dependency (${input.baz[0]}) → `expect(err1).toContain("Saved lockfile")`

## RESP11
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1158` — Response > Response.json > works → `expect(await Response.json(input).text()).toBe(output)`
- `response.spec.md:21` — Response.json static method → `Response.json(data) returns a Response containing the JSON serialization of data`
- `test/js/web/fetch/fetch.test.ts:1161` — Response > Response.json > works → `expect(await Response.json().text()).toBe("")`

## RESP12
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/fetch/response.test.ts:104` —  → `assert(response.bodyUsed)`
- `response.spec.md:9` — Response is exposed as a global constructor → `new Response(body) wraps body as the response body`
- `test/js/deno/fetch/response.test.ts:106` —  → `assert(response.bodyUsed)`

