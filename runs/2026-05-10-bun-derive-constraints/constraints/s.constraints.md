# S — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: s-surface-property
  threshold: S1
  interface: [S]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 22.

## S1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**S** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 1 test files. Antichain representatives:

- `test/bundler/transpiler/decorators.test.ts:454` — decorators random → `expect(S[h]).toBe(30)`
- `test/bundler/transpiler/decorators.test.ts:457` — decorators random → `expect(S[q]).toBe(202)`
- `test/bundler/transpiler/decorators.test.ts:466` — decorators random → `expect(S[u3]).toBe(undefined)`

