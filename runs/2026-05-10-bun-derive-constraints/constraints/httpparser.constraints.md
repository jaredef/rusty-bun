# HTTPParser — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: httpparser-surface-property
  threshold: HTTP1
  interface: [HTTPParser]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 17.

## HTTP1
type: specification
authority: derived
scope: module
status: active
depends-on: []

**HTTPParser** — is defined and resolves to a non-nullish value at the documented call site. (behavioral; cardinality 17)

Witnessed by 17 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/http/node-http-parser.test.ts:11` — HTTPParser.prototype.close > does not double free → `expect(parser.close()).toBeUndefined()`
- `test/js/node/http/node-http-parser.test.ts:12` — HTTPParser.prototype.close > does not double free → `expect(parser.close()).toBeUndefined()`
- `test/js/node/http/node-http-parser.test.ts:22` — HTTPParser.prototype.close > does not segfault calling other methods after close → `expect(parser.close()).toBeUndefined()`

