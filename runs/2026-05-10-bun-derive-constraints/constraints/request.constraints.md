# Request — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: request-surface-property
  threshold: REQU1
  interface: [Request, Request, Request.prototype]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 67.

## REQU1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/web/fetch/fetch.test.ts:1467` — Request > clone → `expect(body.signal).toBeDefined()`
- `test/js/web/fetch/fetch.test.ts:1557` — body nullable → `expect(req.body).toBeNull()`
- `test/js/web/fetch/fetch.test.ts:1562` — body nullable → `expect(req.body).toBeNull()`

## REQU2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/jsc/native-constructor-identity.test.ts:77` — native constructor identity survives ICF > Bun native class constructors remain distinct → `expect(new Request("http://x/")).toBeInstanceOf(Request)`

## REQU3
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request.prototype** — exposes values of the expected type or class. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-fetch.test.js:20` — node-fetch → `expect(Request.prototype).toBeInstanceOf(globalThis.Request)`

## REQU4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 61)

Witnessed by 61 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/2993.test.ts:8` — Request cache option is set correctly → `expect(request.cache).toBe(cache)`
- `test/regression/issue/07001.test.ts:11` — req.body.locked is true after body is consumed → `expect(req.body.locked).toBe(true)`
- `test/regression/issue/04947.test.js:6` — new Request('/') works with node-fetch → `expect(new Request("/").url).toBe("/")`

