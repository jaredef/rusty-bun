# fetch — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: fetch-surface-property
  threshold: FETC1
  interface: [fetch, fetch]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 79.

## FETC1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 68)

Witnessed by 68 constraint clauses across 5 test files. Antichain representatives:

- `tests/unit_node/http_test.ts:189` — [node/http] .writeHead() → `assertEquals(res.status, 404)`
- `tests/unit/websocket_test.ts:442` —  → `assertEquals(r.status, 200)`
- `tests/unit/serve_test.ts:665` —  → `assertEquals(resp.status, 200)`

## FETC2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**fetch** — satisfies the documented invariant. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 4 test files. Antichain representatives:

- `tests/unit_node/http_test.ts:148` — [node/http] chunked response → `assert(res.ok)`
- `tests/unit/serve_test.ts:1850` —  → `assert(resp.body)`
- `tests/unit/http_test.ts:278` —  → `assert(resp.body)`

