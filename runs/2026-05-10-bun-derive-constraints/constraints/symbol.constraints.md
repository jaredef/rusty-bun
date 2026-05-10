# Symbol — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: symbol-surface-property
  threshold: SYMB1
  interface: [Symbol.for, Symbol]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 17.

## SYMB1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**Symbol.for** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 12)

Witnessed by 12 constraint clauses across 2 test files. Antichain representatives:

- `test/js/node/util/util-promisify.test.js:96` — util.promisify > promisify.custom > should register shared promisify symbol → `assert.strictEqual(kCustomPromisifiedSymbol, promisify.custom)`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:1078` — no assertion failures 2 → `assert.strictEqual(inspect(map), "<ref *1> Map(1) { [Circular *1] => 'map' }")`
- `test/js/node/util/node-inspect-tests/parallel/util-inspect.test.js:1080` — no assertion failures 2 → `assert.strictEqual(inspect(map), "<ref *1> Map(1) { [Circular *1] => [Circular *1] }")`

## SYMB2
type: specification
authority: derived
scope: module
status: active
depends-on: []

**Symbol** — exposes values of the expected type or class. (behavioral; cardinality 5)

Witnessed by 5 constraint clauses across 2 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:664` — jest-extended > toBeSymbol() → `expect(Symbol()).toBeSymbol()`
- `test/js/bun/test/expect.test.js:3952` — expect() > toBeSymbol() → `expect(Symbol()).toBeSymbol()`
- `test/js/bun/test/jest-extended.test.js:665` — jest-extended > toBeSymbol() → `expect(Symbol("")).toBeSymbol()`

