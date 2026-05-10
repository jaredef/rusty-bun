# Response — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: response-surface-property
  threshold: RESP1
  interface: [Response]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 9.

## RESP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/response_test.ts:73` —  → `assertEquals(response.status, 200)`
- `tests/unit/fetch_test.ts:1194` —  → `assertEquals(response.status, 200)`
- `tests/unit/fetch_test.ts:1195` —  → `assertEquals(response.body, null)`

