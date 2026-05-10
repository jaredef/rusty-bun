# performance — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: performance-surface-property
  threshold: PERF1
  interface: [performance.mark, performance.getEntriesByType, performance.mark]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 20.

## PERF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/performance_test.ts:101` —  → `assertEquals(mark.detail, null)`
- `tests/unit/performance_test.ts:102` —  → `assertEquals(mark.name, "test")`
- `tests/unit/performance_test.ts:103` —  → `assertEquals(mark.entryType, "mark")`

## PERF2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.getEntriesByType** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/performance_test.ts:49` —  → `assertEquals(performance.getEntriesByType("mark").length, marksNum - 2)`
- `tests/unit/performance_test.ts:52` —  → `assertEquals(performance.getEntriesByType("mark").length, 0)`
- `tests/unit/performance_test.ts:67` —  → `assertEquals(performance.getEntriesByType("measure").length, measuresNum - 2)`

## PERF3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — satisfies the documented invariant. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `tests/unit/performance_test.ts:100` —  → `assert(mark instanceof PerformanceMark)`
- `tests/unit/performance_test.ts:104` —  → `assert(mark.startTime > 0)`
- `tests/unit/performance_test.ts:115` —  → `assert(mark instanceof PerformanceMark)`

