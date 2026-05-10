# platform-cfg/constructor+handle/@js — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: platform-cfg-constructor-handle-js-surface-property
  threshold: PLAT1
  interface: [net.Server.prototype.__proto__]

@imports: []

@pins: []

Surface drawn from 4 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 50.

## PLAT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**net.Server.prototype.__proto__** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 1 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/net/server.spec.ts:82` — net.Server.prototype > has EventEmitter methods → `expect(net.Server.prototype.__proto__).toBe(EventEmitter.prototype)`

## PLAT2
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

## PLAT3
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

## PLAT4
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Intl.DateTimeFormat** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 9)

Witnessed by 9 constraint clauses across 4 test files. Antichain representatives:

- `test/js/node/process/process.test.js:198` — process.env.TZ → `expect(origTimezone).toBe("Etc/UTC")`
- `test/js/bun/test/test-timers.test.ts:18` — we can go back in time → `expect(new Intl.DateTimeFormat().format()).toBe("12/19/1995")`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:6043` — Intl API > Executes formatRange like normal → `assert.equals(new Intl.DateTimeFormat("en-GB", options).formatRange(start, end), "00:00–00:01")`

