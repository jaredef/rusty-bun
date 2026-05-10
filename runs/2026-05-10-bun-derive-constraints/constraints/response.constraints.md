# Response — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: response-surface-property
  threshold: RESP1
  interface: [Response, Response.error, Response, Response.prototype]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 4; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 259.

## RESP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 158 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/19850/19850.test.ts:22` — when beforeEach callback throws > test name is not garbled → `expect(err).toBe(' err-in-hook-and-multiple-tests.ts: 1 | import { beforeEach, test } from "bun:test"; 2 | 3 | beforeEach(() => { 4 | throw new Error("beforeEach"); ^ error: beforeEach at <anonymous> …`
- `test/regression/issue/09555.test.ts:154` — #09555 > Readable.fromWeb consumes the ReadableStream → `expect(response.bodyUsed).toBe(false)`
- `test/regression/issue/02368.test.ts:14` — can clone a response → `expect(await response.text()).toBe("bun")`

## RESP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.error** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1248` — Response > Response.error > works → `expect(Response.error().type).toBe("error")`
- `test/js/web/fetch/fetch.test.ts:1249` — Response > Response.error > works → `expect(Response.error().ok).toBe(false)`
- `test/js/web/fetch/fetch.test.ts:1250` — Response > Response.error > works → `expect(Response.error().status).toBe(0)`

## RESP3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:79` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Response("")).toBeInstanceOf(Response)`

## RESP4
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Response.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-fetch.test.js:19` — node-fetch → `expect(Response.prototype).toBeInstanceOf(globalThis.Response)`

## RESP5
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Response** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 76)

Witnessed by 76 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/fetch/utf8-bom.test.ts:7` — UTF-8 BOM should be ignored > handles empty strings → `expect(await blob.text()).toHaveLength(0)`
- `test/js/web/fetch/body.test.ts:629` — new Response() → `expect(result).toHaveProperty("bodyUsed", false)`
- `test/cli/update_interactive_formatting.test.ts:789` — bun update --interactive > should handle catalog updates in workspaces.catalogs object → `expect(output).toContain("Installing updates...")`

## RESP6
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.json** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1158` — Response > Response.json > works → `expect(await Response.json(input).text()).toBe(output)`
- `test/js/web/fetch/fetch.test.ts:1161` — Response > Response.json > works → `expect(await Response.json().text()).toBe("")`
- `test/js/web/fetch/fetch.test.ts:1163` — Response > Response.json > works → `expect(await Response.json("").text()).toBe('""')`

## RESP7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/07397.test.ts:9` — Response.redirect clones string from Location header → `expect(response.headers.get("Location")).toBe(href)`
- `test/js/web/fetch/response.test.ts:99` — Response.redirect status code validation → `expect(Response.redirect("url", 301).status).toBe(301)`
- `test/js/web/fetch/fetch.test.ts:1226` — Response > Response.redirect > works → `expect(Response.redirect(input).headers.get("Location")).toBe(input)`

