# performance — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: performance-surface-property
  threshold: PERF1
  interface: [performance.now, performance.mark, performance.measure]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 3; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 32.

## PERF1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.now** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 12 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/timers/setInterval.test.js:31` — setInterval → `expect(performance.now() - start > 9).toBe(true)`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:300` — performance.now() mocking > performance.now() should be mocked when fake timers are active → `expect(performance.now()).toBe(1000)`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:304` — performance.now() mocking > performance.now() should be mocked when fake timers are active → `expect(performance.now()).toBe(1500)`

## PERF2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:37` —  → `assertEquals(mark.detail, null)`
- `test/js/deno/performance/performance.test.ts:38` —  → `assertEquals(mark.name, "test")`
- `test/js/deno/performance/performance.test.ts:39` —  → `assertEquals(mark.entryType, "mark")`

## PERF3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.measure** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 4 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:86` —  → `assertEquals(measure1.detail, null)`
- `test/js/deno/performance/performance.test.ts:87` —  → `assertEquals(measure1.name, measureName1)`
- `test/js/deno/performance/performance.test.ts:88` —  → `assertEquals(measure1.entryType, "measure")`

## PERF4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**performance.now** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/timers/performance.test.js:24` — performance.now() should be monotonic → `expect(first).toBeLessThanOrEqual(second)`
- `test/js/bun/util/sleep.test.ts:45` — sleep should saturate timeout values → `expect(performance.now() - start).toBeLessThan(1000 * ASAN_MULTIPLIER)`
- `test/js/bun/cron/cron-parse.test.ts:50` — Bun.cron.parse — UTC > impossible day/month (Feb 30) returns null quickly → `expect(performance.now() - t).toBeLessThan(50)`

