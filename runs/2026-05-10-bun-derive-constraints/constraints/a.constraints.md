# A — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: a-surface-property
  threshold: A1
  interface: [A]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 20.

## A1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**A** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 20)

Witnessed by 20 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:1807` — expect() > toEqual() - private class fields → `expect(a1).toEqual(a2)`
- `test/bundler/transpiler/decorators.test.ts:633` — class extending from another class → `expect(new A().a).toBe(3)`
- `src/bun_core/output.rs:2861` — ln_macros_suppress_double_newline → `assert_eq!(A , "")`

