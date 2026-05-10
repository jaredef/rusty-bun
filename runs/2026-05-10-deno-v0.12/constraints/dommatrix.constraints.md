# DOMMatrix — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: dommatrix-surface-property
  threshold: DOMM1
  interface: [DOMMatrix.fromMatrix]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 27.

## DOMM1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**DOMMatrix.fromMatrix** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 27)

Witnessed by 27 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/geometry_test.ts:44` —  → `assertEquals( matrix, DOMMatrix.fromMatrix(init), )`
- `tests/unit/geometry_test.ts:74` —  → `assertEquals( matrix, // deno-fmt-ignore DOMMatrix.fromMatrix({ m11: 1, m21: 2, m31: 3, m41: 1 * 1 + 2 * 2 + 3 * 3 + 4 * 1, m12: 5, m22: 6, m32: 7, m42: 5 * 1 + 6 * 2 + 7 * 3 + 8 * 1, m13: 9, m23: 10,…`
- `tests/unit/geometry_test.ts:96` —  → `assertEquals( matrix, DOMMatrix.fromMatrix(init), )`

