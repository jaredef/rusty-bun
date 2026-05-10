# setTimeout — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: settimeout-surface-property
  threshold: SETT1
  interface: [setTimeout]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 13.

## SETT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**setTimeout** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/25639.test.ts:12` — setTimeout returns Timeout object with _idleStart property → `expect(typeof timer._idleStart).toBe("number")`
- `test/regression/issue/25639.test.ts:46` — _idleStart is writable (Next.js modifies it to coordinate timers) → `expect(timer._idleStart).toBe(newIdleStart)`
- `test/js/web/timers/setTimeout.test.js:301` — setTimeout should refresh N times → `expect(timer.refresh()).toBe(timer)`

## SETT2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**setTimeout** — satisfies the documented invariant. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3437` — FakeTimers > stubTimers > global fake setTimeout should return id → `assert.isFunction(to.ref)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3438` — FakeTimers > stubTimers > global fake setTimeout should return id → `assert.isFunction(to.unref)`
- `test/js/bun/test/fake-timers/sinonjs/fake-timers.test.ts:3440` — FakeTimers > stubTimers > global fake setTimeout should return id → `assert.isNumber(to)`

