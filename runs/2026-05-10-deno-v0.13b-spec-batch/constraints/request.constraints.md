# Request — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: request-surface-property
  threshold: REQU1
  interface: [Request, Request, Request.prototype.clone]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 12.

## REQU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 3 constraint clauses across 1 test files. Antichain representatives:

- `request.spec.md:12` — Request constructor validates init → `Request constructor throws TypeError on invalid URL`
- `request.spec.md:13` — Request constructor validates init → `Request constructor throws TypeError when init.method is a forbidden method`
- `request.spec.md:14` — Request constructor validates init → `Request constructor throws TypeError when init.mode is "navigate"`

## REQU2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Request** — is defined and resolves to a non-nullish value at the documented call site. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `request.spec.md:7` — Request is exposed as a global constructor → `Request is defined as a global constructor in any execution context with [Exposed=*]`

## REQU3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request.prototype.clone** — throws or rejects with a documented error shape on invalid inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `request.spec.md:59` — Request.prototype.clone → `Request.prototype.clone throws TypeError when bodyUsed is true`

## REQU4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/request_test.ts:14` —  → `assertEquals(req.url, "http://foo/")`
- `tests/unit/fetch_test.ts:2039` —  → `assertEquals(req.method, "POST")`
- `tests/unit/request_test.ts:26` —  → `assertEquals(new Request(nonString).url, "http://foo/")`

