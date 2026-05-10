# returns-error/success-errors/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: returns-error-success-errors-regression-surface-property
  threshold: RETU1
  interface: [Bun.build]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 182.

## RETU1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Bun.build** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 182 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/5344.test.ts:19` — code splitting with re-exports between entry points should not produce duplicate exports → `expect(result.success).toBe(true)`
- `test/regression/issue/26360.test.ts:135` — regular Bun.build (not in macro) still works → `expect(result.success).toBe(true)`
- `test/regression/issue/25785.test.ts:35` — CSS bundler should preserve logical border-radius properties → `expect(result.success).toBe(true)`

