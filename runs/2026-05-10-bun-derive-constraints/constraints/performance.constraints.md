# performance — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: performance-surface-property
  threshold: PERF1
  interface: [performance.now]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 54.

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
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**performance.now** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 25)

Witnessed by 25 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/timers/performance.test.js:24` — performance.now() should be monotonic → `expect(first).toBeLessThanOrEqual(second)`
- `test/js/web/fetch/fetch.tls.test.ts:340` — fetch-tls > fetch timeout works on tls → `expect(total).toBeGreaterThanOrEqual(TIMEOUT - THRESHOLD)`
- `test/js/web/fetch/fetch-tls-abortsignal-timeout.test.ts:30` — fetch should abort as soon as possible under tls using AbortSignal.timeout(${timeout}) → `expect(diff).toBeLessThanOrEqual(timeout + THRESHOLD)`

## PERF3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:37` —  → `assertEquals(mark.detail, null)`
- `test/js/deno/performance/performance.test.ts:38` —  → `assertEquals(mark.name, "test")`
- `test/js/deno/performance/performance.test.ts:39` —  → `assertEquals(mark.entryType, "mark")`

## PERF4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**performance.mark** — satisfies the documented invariant. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:36` —  → `assert(mark instanceof PerformanceMark)`
- `test/js/deno/performance/performance.test.ts:40` —  → `assert(mark.startTime > 0)`
- `test/js/deno/performance/performance.test.ts:51` —  → `assert(mark instanceof PerformanceMark)`

