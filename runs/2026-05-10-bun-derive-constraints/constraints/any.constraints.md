# ANY — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: any-surface-property
  threshold: ANY1
  interface: [ANY]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 21.

## ANY1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ANY** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:1080` — expect() > deepEquals large object → `expect(o).toEqual(o)`
- `test/js/bun/test/expect.test.js:1085` — expect() > deepEquals large object → `expect(b).toEqual(b)`
- `test/js/bun/test/expect.test.js:1113` — expect() > deepEquals large object → `expect(p).toEqual(p)`

