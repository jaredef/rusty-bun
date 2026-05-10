# Number — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: number-surface-property
  threshold: NUMB1
  interface: [Number, Number, Number.isNaN, Number, Number.isNaN, Number.MAX_VALUE]

@imports: []

@pins: []

Surface drawn from 6 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 6. Total witnessing constraint clauses: 64.

## NUMB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Number** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/26030.test.ts:71` — Sequential transactions with INSERT and returned SELECT should not hang → `expect(Number(count[0].count)).toBe(3)`
- `test/js/third_party/jsonwebtoken/verify.test.js:213` — verify > expiration > should error on expired token → `expect(Number(err.expiredAt)).toBe(1437018592000)`
- `test/js/sql/sql.test.ts:4787` — PostgreSQL tests > int8 Array Type ${bigint ? " (BigInt)" : ""} > int8[] - array mathemati… → `expect(Number(result[0].average)).toBe(2)`

## NUMB2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Number** — satisfies the documented invariant. (behavioral; cardinality 19)

Witnessed by 19 constraint clauses across 3 test files. Antichain representatives:

- `test/regression/issue/24045.test.ts:7` — assert.deepStrictEqual() should compare Number wrapper object values - issue #24045 → `assert.deepStrictEqual(new Number(1), new Number(2))`
- `test/js/node/child_process/child-process-rlimit-nofile.test.ts:35` — child process inherits a sane RLIMIT_NOFILE (capped at 1<<20) → `assert.ok(n > 256, 'runtime should raise the soft limit above 256, got ${n}')`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:68` — FakeTimers > setTimeout > returns numeric id or object with numeric id → `assert.isNumber(Number(result))`

## NUMB3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Number.isNaN** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 3 test files. Antichain representatives:

- `test/js/web/fetch/content-length.test.js:9` — Content-Length is set when using a FormData body with fetch → `expect(Number.isNaN(Number(req.headers["content-length"]))).toBe(false)`
- `test/js/bun/json5/json5.test.ts:178` — numbers - additional > +NaN → `expect(Number.isNaN(parsed)).toBe(true)`
- `test/js/bun/json5/json5-test-suite.test.ts:551` — numbers > nan → `expect(Number.isNaN(parsed)).toBe(true)`

## NUMB4
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Number** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 4 test files. Antichain representatives:

- `test/js/bun/webview/webview.test.ts:655` — scrollTo(selector, { block }) controls alignment → `expect(topStart).toBeLessThan(20)`
- `test/js/bun/http/serve-http3.test.ts:1009` — Bun.serve HTTP/3 lifecycle > req.signal aborts on client RST → `expect(Number(count)).toBeGreaterThan(0)`
- `test/cli/test/parallel.test.ts:199` — --parallel --bail stops dispatching new files after threshold → `expect(Number(m![1])).toBeLessThanOrEqual(2)`

## NUMB5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Number.isNaN** — satisfies the documented invariant. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/perf_hooks/histogram.test.ts:16` — Histogram > basic histogram creation and initial state → `assert.ok(Number.isNaN(h.mean))`
- `test/js/bun/perf_hooks/histogram.test.ts:17` — Histogram > basic histogram creation and initial state → `assert.ok(Number.isNaN(h.stddev))`
- `test/js/bun/perf_hooks/histogram.test.ts:115` — Histogram > reset functionality → `assert.ok(Number.isNaN(h.mean))`

## NUMB6
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Number.MAX_VALUE** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:2967` — expect() > toBeGreaterThan() → `expect(Number.MAX_VALUE).toBeGreaterThan(1n)`
- `test/js/bun/test/expect.test.js:2980` — expect() > toBeGreaterThanOrEqual() → `expect(Number.MAX_VALUE).toBeGreaterThanOrEqual(Number.MAX_VALUE)`
- `test/js/bun/test/expect.test.js:3078` — expect() > toBeGreaterThanOrEqual() → `expect(Number.MAX_VALUE).toBeGreaterThanOrEqual(1n)`

