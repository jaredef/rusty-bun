# WebAssembly — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: webassembly-surface-property
  threshold: WEBA1
  interface: [WebAssembly.Global]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 16.

## WEBA1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**WebAssembly.Global** — exhibits the property captured in the witnessing test. (behavioral; cardinality 16)

Witnessed by 16 constraint clauses across 1 test files. Antichain representatives:

- `test/js/bun/test/jest-extended.test.js:410` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i32", mutable: false }, 4).value).toBeEven()`
- `test/js/bun/test/jest-extended.test.js:412` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i32", mutable: true }, 2).value).toBeEven()`
- `test/js/bun/test/jest-extended.test.js:415` — jest-extended > toBeEven() → `expect(new WebAssembly.Global({ value: "i64", mutable: true }, -9223372036854775808n).value).toBeEven()`

