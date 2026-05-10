# Stats — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: stats-surface-property
  threshold: STAT1
  interface: [Stats, Stats]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 36.

## STAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Stats** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 22)

Witnessed by 22 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs.test.ts:3391` — new Stats → `expect(stats.dev).toBe(1)`
- `test/js/node/fs/fs.test.ts:3392` — new Stats → `expect(stats.mode).toBe(2)`
- `test/js/node/fs/fs.test.ts:3393` — new Stats → `expect(stats.nlink).toBe(3)`

## STAT2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Stats** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 14)

Witnessed by 14 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/fs/fs-stats-truncate.test.ts:12` — fs.stats truncate → `expect(stats.dev).toBeGreaterThan(0)`
- `test/js/node/fs/fs-stats-truncate.test.ts:13` — fs.stats truncate → `expect(stats.mode).toBeGreaterThan(0)`
- `test/js/node/fs/fs-stats-truncate.test.ts:14` — fs.stats truncate → `expect(stats.nlink).toBeGreaterThan(0)`

