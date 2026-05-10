# Response — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: response-surface-property
  threshold: RESP1
  interface: [Response, Response.redirect]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 19.

## RESP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `tests/unit/response_test.ts:73` —  → `assertEquals(response.status, 200)`
- `tests/unit/fetch_test.ts:994` —  → `assertEquals(await response.bytes(), new Uint8Array(0))`
- `tests/unit/fetch_test.ts:998` —  → `assertEquals(await response.text(), "")`

## RESP2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Response.redirect** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/fetch_test.ts:974` —  → `assertEquals(redir.status, 301)`
- `tests/unit/fetch_test.ts:975` —  → `assertEquals(redir.statusText, "")`
- `tests/unit/fetch_test.ts:976` —  → `assertEquals(redir.url, "")`

