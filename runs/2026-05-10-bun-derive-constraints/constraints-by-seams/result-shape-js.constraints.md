# result-shape/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: result-shape-js-surface-property
  threshold: RESU1
  interface: [Response.error]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 3.

## RESU1
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

