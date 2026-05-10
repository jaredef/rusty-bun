# StringDecoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: stringdecoder-surface-property
  threshold: STRI1
  interface: [StringDecoder]

@imports: []

@pins: []

Surface drawn from 1 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 1. Total witnessing constraint clauses: 23.

## STRI1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**StringDecoder** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 23)

Witnessed by 23 constraint clauses across 1 test files. Antichain representatives:

- `test/js/node/string_decoder/string-decoder.test.js:51` — StringDecoder-utf8-additional → `expect(decoder.write(Buffer.from("E18B", "hex"))).toBe("")`
- `test/js/node/string_decoder/string-decoder.test.js:52` — StringDecoder-utf8-additional → `expect(decoder.end()).toBe("\ufffd")`
- `test/js/node/string_decoder/string-decoder.test.js:55` — StringDecoder-utf8-additional → `expect(decoder.write(Buffer.from("\ufffd"))).toBe("\ufffd")`

