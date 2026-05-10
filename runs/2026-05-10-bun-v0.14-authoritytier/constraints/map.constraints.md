# Map — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: map-surface-property
  threshold: MAP1
  interface: [Map, Map]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 16.

## MAP1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Map** — satisfies the documented invariant. (behavioral; cardinality 10)

Witnessed by 10 constraint clauses across 1 test files. Antichain representatives:

- `test/regression/issue/24338.test.ts:9` — partialDeepStrictEqual with Map subset - basic case → `assert.partialDeepStrictEqual( new Map([ ["key1", "value1"], ["key2", "value2"], ]), new Map([["key2", "value2"]]), )`
- `test/regression/issue/24338.test.ts:19` — partialDeepStrictEqual with Map - exact match → `assert.partialDeepStrictEqual(new Map([["key1", "value1"]]), new Map([["key1", "value1"]]))`
- `test/regression/issue/24338.test.ts:23` — partialDeepStrictEqual with Map - empty expected → `assert.partialDeepStrictEqual(new Map([["key1", "value1"]]), new Map())`

## MAP2
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

