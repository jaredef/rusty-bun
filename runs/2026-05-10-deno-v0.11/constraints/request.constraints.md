# Request — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: request-surface-property
  threshold: REQU1
  interface: [Request]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 14.

## REQU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/request_test.ts:14` —  → `assertEquals(req.url, "http://foo/")`
- `tests/unit/fetch_test.ts:2032` —  → `assertEquals(await req.text(), "foo")`
- `tests/unit/request_test.ts:15` —  → `assertEquals(req.headers.get("test-header"), "value")`

