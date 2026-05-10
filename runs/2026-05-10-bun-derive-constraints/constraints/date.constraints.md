# Date — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: date-surface-property
  threshold: DATE1
  interface: [Date, Date.now, Date.now, Date.now, Date, Date, Date.parse]

@imports: []

@pins: []

Surface drawn from 7 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 7. Total witnessing constraint clauses: 108.

## DATE1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Date** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 29)

Witnessed by 29 constraint clauses across 5 test files. Antichain representatives:

- `test/js/sql/sqlite-sql.test.ts:811` — Data Types & Values > handles Date values (stored as TEXT) → `expect(new Date(result[0].value)).toEqual(date)`
- `test/js/sql/sql.test.ts:299` — PostgreSQL tests > Array helpers > sql.array should support TIMESTAMP arrays → `expect(new Date(x[0])).toEqual(ts1)`
- `test/js/node/v8/v8-date-parser.test.js:51` — v8 date parser > test/webkit/date-parse-comments-test.js → `expect(new Date(date).toString()).toBe(new Date(numericResult).toString())`

## DATE2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Date.now** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 21)

Witnessed by 21 constraint clauses across 3 test files. Antichain representatives:

- `test/js/bun/test/test-timers.test.ts:9` — we can go back in time → `expect(Date.now()).toBe(819331200000)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3751` — FakeTimers > stubTimers > decide on Date.now support at call-time when supported → `assert.equals(typeof Date.now, "function")`
- `test/js/bun/test/fake-timers/fake-timers.test.ts:235` — Date.now() mocking > Date.now() should be mocked when fake timers are active → `expect(Date.now()).toBe(start + 1000)`

## DATE3
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Date.now** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 20)

Witnessed by 20 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/timers/setTimeout.test.js:318` — setTimeout if refreshed before run, should reschedule to run later → `expect(end - start).toBeGreaterThan(120)`
- `test/js/sql/sqlite-sql.test.ts:3158` — Concurrency and Locking > write lock prevents concurrent writes → `expect(duration).toBeGreaterThanOrEqual(40)`
- `test/js/node/http/node-http.test.ts:1780` — HTTP Server Security Tests - Advanced > flushHeaders should send the headers immediately → `expect(Date.now() - headers_sent_at).toBeLessThan(100)`

## DATE4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Date.now** — satisfies the documented invariant. (behavioral; cardinality 13)

Witnessed by 13 constraint clauses across 2 test files. Antichain representatives:

- `test/js/deno/performance/performance.test.ts:22` —  → `assert(Date.now() >= origin)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3758` — FakeTimers > stubTimers > decide on Date.now support at call-time when unsupported → `assert.isUndefined(Date.now)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3819` — FakeTimers > shouldAdvanceTime > should create an auto advancing timer → `assert.same(Date.now(), 1443139200000)`

## DATE5
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Date** — exhibits the property captured in the witnessing test. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/v8/v8-date-parser.test.js:46` — v8 date parser > test/webkit/date-parse-comments-test.js → `expect(new Date(date).getMilliseconds()).toBeNaN()`
- `test/js/bun/test/expect.test.js:3981` — expect() > toBeValidDate() → `expect(new Date()).toBeValidDate()`
- `test/js/node/v8/v8-date-parser.test.js:115` — v8 date parser > test/mjsunit/regress-4640.js → `expect(new Date("275760-10-14").getMilliseconds()).toBeNaN()`

## DATE6
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Date** — exposes values of the expected type or class. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:258` — jest-extended > toBeDate() → `expect(new Date()).toBeDate()`
- `test/js/bun/test/expect.test.js:3972` — expect() > toBeDate() → `expect(new Date()).toBeDate()`
- `test/js/bun/test/jest-extended.test.js:259` — jest-extended > toBeDate() → `expect(new Date(0)).toBeDate()`

## DATE7
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Date.parse** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/v8/v8-date-parser.test.js:48` — v8 date parser > test/webkit/date-parse-comments-test.js → `expect(Date.parse(date)).toBe(numericResult)`
- `test/js/node/v8/v8-date-parser.test.js:49` — v8 date parser > test/webkit/date-parse-comments-test.js → `expect(Date.parse(date.toUpperCase())).toBe(numericResult)`
- `test/js/node/v8/v8-date-parser.test.js:50` — v8 date parser > test/webkit/date-parse-comments-test.js → `expect(Date.parse(date.toLowerCase())).toBe(numericResult)`

