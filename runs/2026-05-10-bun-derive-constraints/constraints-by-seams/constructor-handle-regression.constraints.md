# constructor+handle/@regression — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: constructor-handle-regression-surface-property
  threshold: CONS1
  interface: [Request, Map, Object.prototype.hasOwnProperty.call]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 79.

## CONS1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Request** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 61)

Witnessed by 61 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/2993.test.ts:8` — Request cache option is set correctly → `expect(request.cache).toBe(cache)`
- `test/regression/issue/07001.test.ts:11` — req.body.locked is true after body is consumed → `expect(req.body.locked).toBe(true)`
- `test/regression/issue/04947.test.js:6` — new Request('/') works with node-fetch → `expect(new Request("/").url).toBe("/")`

## CONS2
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Map** — satisfies the documented invariant. (behavioral; cardinality 11)

Witnessed by 11 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24338.test.ts:9` — partialDeepStrictEqual with Map subset - basic case → `assert.partialDeepStrictEqual( new Map([ ["key1", "value1"], ["key2", "value2"], ]), new Map([["key2", "value2"]]), )`
- `test/regression/issue/24338.test.ts:19` — partialDeepStrictEqual with Map - exact match → `assert.partialDeepStrictEqual(new Map([["key1", "value1"]]), new Map([["key1", "value1"]]))`
- `test/regression/issue/24338.test.ts:23` — partialDeepStrictEqual with Map - empty expected → `assert.partialDeepStrictEqual(new Map([["key1", "value1"]]), new Map())`

## CONS3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Object.prototype.hasOwnProperty.call** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 7)

Witnessed by 7 constraint clauses across 2 test files. Antichain representatives:

- `test/regression/issue/26284.test.ts:24` — hasOwnProperty('clock') returns false before useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(false)`
- `test/regression/issue/25869.test.ts:22` — setTimeout.clock is not set before useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(false)`
- `test/regression/issue/26284.test.ts:30` — hasOwnProperty('clock') returns true after useFakeTimers → `expect(Object.prototype.hasOwnProperty.call(globalThis.setTimeout, "clock")).toBe(true)`

