# TextDecoder — surface constraints derived from the Bun test corpus

*Auto-drafted from `derive-constraints invert` over the bun test corpus extraction at rusty-bun/runs/2026-05-10-bun-derive-constraints. This file is a draft constraint document in the [rederive grammar](https://github.com/jaredef/rederive). The substrate at rederive's derive step interprets the prose into target-language code; this draft is keeper-authorable scaffold, not final spec. See [Doc 704 (The 'Port' as Translation Is a Category Error)](https://jaredfoy.com/resolve/doc/704-the-port-as-translation-is-a-category-error) for the apparatus this output serves.*

@provides: textdecoder-surface-property
  threshold: TEXT1
  interface: [TextDecoder, TextDecoder]

@imports: []

@pins: []

Surface drawn from 2 candidate properties across the Bun test corpus. Construction-style: 0; behavioral (high-cardinality): 2. Total witnessing constraint clauses: 191.

## TEXT1
type: predicate
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — produces values matching the documented patterns under the documented inputs. (behavioral; cardinality 157)

Witnessed by 157 constraint clauses across 5 test files. Antichain representatives:

- `test/js/web/streams/streams.test.js:611` — readableStreamToArrayBuffer (bytes) → `expect(new TextDecoder().decode(new Uint8Array(buffer))).toBe("abdefgh")`
- `test/js/web/encoding/text-encoder.test.js:158` — TextEncoder > should encode long latin1 text → `expect(decoded).toBe(text)`
- `test/js/web/encoding/text-decoder.test.js:23` — TextDecoder > should not crash on empty text → `expect(decoder.decode(input)).toBe("")`

## TEXT2
type: invariant
authority: derived
scope: module
status: active
depends-on: []

**TextDecoder** — satisfies the documented containment / structural-shape invariants. (behavioral; cardinality 34)

Witnessed by 34 constraint clauses across 5 test files. Antichain representatives:

- `test/regression/issue/pe-codesigning-integrity.test.ts:241` — should create valid PE executable with .bun section → `expect(embeddedText).toContain("B:/~BUN/root/")`
- `test/regression/issue/23474.test.ts:40` — request.cookies.set() should set websocket upgrade response cookie - issue #23474 → `expect(response).toContain("HTTP/1.1 101")`
- `test/js/web/fetch/body-clone.test.ts:428` — ReadableStream with mixed content (starting with Uint8Array) can be converted to ArrayBuff… → `expect(text).toContain("Hello")`

