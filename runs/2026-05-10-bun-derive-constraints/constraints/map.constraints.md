# Map — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: map-surface-property
  threshold: MAP1
  interface: [Map, Map, Map]

@imports: []

@pins: []

Surface drawn from 3 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 3. Total witnessing constraint clauses: 25.

## MAP1
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

## MAP2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**Map** — satisfies the documented ordering / proximity invariants. (behavioral; cardinality 8)

Witnessed by 8 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/util/heap-snapshot.test.ts:25` — Native types report their size correctly > FormData → `expect(summariesMap.get("FormData")?.size).toBeGreaterThan( // Test that FormData includes the size of the strings and the blobs 1024 * 1024 * 8 + 1024 * 1024 * 2 + 1024 * 1024 * 2, )`
- `test/js/bun/util/heap-snapshot.test.ts:44` — Native types report their size correctly > Request → `expect(summariesMap.get("Request")?.size).toBeGreaterThan(1024 * 1024 * 2)`
- `test/js/bun/util/heap-snapshot.test.ts:45` — Native types report their size correctly > Request → `expect(summariesMap.get("Request")?.size).toBeLessThan(1024 * 1024 * 4)`

## MAP3
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Map** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 6)

Witnessed by 6 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/expect.test.js:1237` — expect() > deepEquals set and map → `expect(e).toEqual(d)`
- `test/js/bun/jsc/native-constructor-identity.test.ts:38` — native constructor identity survives ICF > expect.any distinguishes builtin constructors w… → `expect(new Map()).toEqual(expect.any(Map))`
- `test/js/bun/test/expect.test.js:1238` — expect() > deepEquals set and map → `expect(d).toEqual(e)`

