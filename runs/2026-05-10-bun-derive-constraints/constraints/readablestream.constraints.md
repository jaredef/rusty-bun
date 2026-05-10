# ReadableStream — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: readablestream-surface-property
  threshold: READ1
  interface: [ReadableStream]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 1; behavioral (high-cardinality): 0. Total witnessing constraint clauses: 3.

## READ1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**ReadableStream** — produces values matching the documented patterns under the documented inputs. (construction-style)

Witnessed by 3 constraint clauses across 2 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:472` — exists globally → `expect(typeof ReadableStream).toBe("function")`
- `test/js/web/fetch/utf8-bom.test.ts:131` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.text() → `expect(await stream.text()).toBe("Hello, World!")`
- `test/js/web/fetch/utf8-bom.test.ts:141` — UTF-8 BOM should be ignored > readable stream > in ReadableStream.prototype.json() → `expect(await stream.json()).toEqual({ "hello": "World" })`

